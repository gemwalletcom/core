use crate::network::AlienProvider;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

mod custom_types;
mod permit2_data;

pub mod jupiter;
pub mod models;
pub mod orca;
pub mod slippage;
pub mod thorchain;
pub mod universal_router;

pub use models::*;
use primitives::Chain;
use std::collections::HashSet;

#[async_trait]
pub trait GemSwapProvider: Send + Sync + Debug {
    fn provider(&self) -> SwapProvider;
    fn supported_chains(&self) -> Vec<Chain>;
    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError>;
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError>;
    async fn get_transaction_status(&self, chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError>;
}

#[derive(Debug, uniffi::Object)]
pub struct GemSwapper {
    pub rpc_provider: Arc<dyn AlienProvider>,
    pub swappers: Vec<Box<dyn GemSwapProvider>>,
}

#[uniffi::export]
impl GemSwapper {
    #[uniffi::constructor]
    fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            rpc_provider,
            swappers: vec![
                Box::new(universal_router::UniswapV3::new_uniswap()),
                Box::new(universal_router::UniswapV3::new_pancakeswap()),
                Box::new(thorchain::ThorChain::default()),
                Box::new(jupiter::Jupiter::default()),
            ],
        }
    }

    fn supported_chains(&self) -> Vec<Chain> {
        self.swappers
            .iter()
            .flat_map(|x| x.supported_chains())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn get_providers(&self) -> Vec<SwapProvider> {
        self.swappers.iter().map(|x| x.provider()).collect()
    }

    async fn fetch_quote(&self, request: SwapQuoteRequest) -> Result<Vec<SwapQuote>, SwapperError> {
        if request.from_asset == request.to_asset {
            return Err(SwapperError::NotSupportedPair);
        }

        let providers = self
            .swappers
            .iter()
            .filter(|x| {
                let supported_chains = x.supported_chains();
                supported_chains.contains(&request.from_asset.chain) && supported_chains.contains(&request.to_asset.chain)
            })
            .collect::<Vec<_>>();

        if providers.is_empty() {
            return Err(SwapperError::NotSupportedPair);
        }

        let quotes_futures = providers.into_iter().map(|x| x.fetch_quote(&request, self.rpc_provider.clone()));

        let quotes = futures::future::join_all(quotes_futures.into_iter().map(|fut| async { fut.await.ok() }))
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        Ok(quotes)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let swapper = self
            .swappers
            .iter()
            .find(|x| x.provider() == quote.data.provider)
            .ok_or(SwapperError::NotImplemented)?;
        swapper.fetch_quote_data(quote, self.rpc_provider.clone(), data).await
    }

    async fn get_transaction_status(&self, chain: Chain, swap_provider: SwapProvider, transaction_hash: &str) -> Result<bool, SwapperError> {
        let swapper = self
            .swappers
            .iter()
            .find(|x| x.provider() == swap_provider)
            .ok_or(SwapperError::NotImplemented)?;

        swapper.get_transaction_status(chain, transaction_hash, self.rpc_provider.clone()).await
    }
}
