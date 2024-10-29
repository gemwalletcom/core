use crate::debug_println;
use crate::network::AlienProvider;

use async_trait::async_trait;
use std::{fmt::Debug, sync::Arc};

mod custom_types;
mod models;
mod slippage;
mod uniswap;
use models::*;

#[async_trait]
pub trait GemSwapProvider: Send + Sync + Debug {
    async fn fetch_quote(&self, request: &GemSwapRequest, provider: Arc<dyn AlienProvider>) -> Result<GemSwapQuote, GemSwapperError>;
    async fn fetch_quote_data(
        &self,
        request: &GemSwapRequest,
        quote: &GemSwapQuote,
        provider: Arc<dyn AlienProvider>,
        approval: Option<GemApprovalData>,
    ) -> Result<GemSwapQuoteData, GemSwapperError>;
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
            swappers: vec![Box::new(uniswap::UniswapV3::new())],
        }
    }

    async fn fetch_quote(&self, request: GemSwapRequest) -> Result<GemSwapQuote, GemSwapperError> {
        for swapper in self.swappers.iter() {
            let quote = swapper.fetch_quote(&request, self.rpc_provider.clone()).await;
            match quote {
                Ok(quote) => return Ok(quote),
                Err(err) => {
                    debug_println!("<== fetch_quote error: {:?}", err);
                }
            }
        }
        Err(GemSwapperError::NoQuoteAvailable)
    }
}
