use crate::debug_println;
use crate::network::AlienProvider;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

mod custom_types;
mod permit2_data;

pub mod models;
pub mod orca;
pub mod slippage;
pub mod thorchain;
pub mod uniswap;

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
                Box::new(uniswap::UniswapV3::new()),
                Box::new(thorchain::ThorChain::new()),
                Box::new(orca::Orca::default()),
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

        for swapper in self.swappers.iter() {
            let quotes = swapper.fetch_quote(&request, self.rpc_provider.clone()).await;
            match quotes {
                Ok(val) => return Ok(vec![val]),
                Err(_err) => {
                    debug_println!("<== fetch_quote error: {:?}", _err);
                }
            }
        }
        Err(SwapperError::NoQuoteAvailable)
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
