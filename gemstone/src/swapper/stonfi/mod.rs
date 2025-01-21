use std::sync::Arc;

use async_trait::async_trait;
use primitives::Chain;

use crate::{network::AlienProvider, swapper::SwapChainAsset};

use super::{ApprovalType, FetchQuoteData, GemSwapProvider, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapperError};

pub mod router;

#[derive(Debug, Default)]
pub struct Stonfi {}

#[async_trait]
impl GemSwapProvider for Stonfi {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Stonfi
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        // TODO: Specify assets
        vec![SwapChainAsset::All(Chain::Ton)]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        println!("Fetch quote for {:?}", request);
        // Return fake quote for testing
        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: request.value.clone(),
            data: SwapProviderData {
                provider: SwapProvider::Stonfi,
                suggested_slippage_bps: Some(100),
                routes: vec![],
            },
            approval: ApprovalType::None,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }

    async fn get_transaction_status(&self, chain: primitives::Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        todo!()
    }
}
