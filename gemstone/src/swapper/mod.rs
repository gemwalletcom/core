use crate::debug_println;
use crate::network::AlienProvider;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

mod custom_types;
mod models;
mod permit2_data;
mod slippage;
mod thorchain;
mod uniswap;

use models::*;
use primitives::Chain;

#[async_trait]
pub trait GemSwapProvider: Send + Sync + Debug {
    fn name(&self) -> &'static str;
    async fn supported_chains(&self) -> Result<Vec<Chain>, SwapperError>;
    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError>;
    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError>;
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
            swappers: vec![Box::new(uniswap::UniswapV3::new()), Box::new(thorchain::ThorChain::new())],
        }
    }

    fn get_providers(&self) -> Vec<String> {
        self.swappers.iter().map(|x| x.name().to_string()).collect()
    }

    async fn fetch_quote(&self, request: SwapQuoteRequest) -> Result<Vec<SwapQuote>, SwapperError> {
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
            .find(|x| x.name() == quote.provider.name.as_str())
            .ok_or(SwapperError::NotImplemented)?;
        swapper.fetch_quote_data(quote, self.rpc_provider.clone(), data).await
    }
}
