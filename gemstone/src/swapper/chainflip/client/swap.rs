use super::{
    model::{QuoteRequest, QuoteResponse},
    SwapTxResponse,
};
use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use std::sync::Arc;

const CHAINFLIP_API_URL: &str = "https://chainflip-swap.chainflip.io";

#[derive(Debug)]
pub struct ChainflipClient {
    provider: Arc<dyn AlienProvider>,
}

impl ChainflipClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Vec<QuoteResponse>, SwapperError> {
        let query = serde_urlencoded::to_string(request).map_err(SwapperError::from)?;
        let url = format!("{CHAINFLIP_API_URL}/v2/quote?{query}");
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await.map_err(SwapperError::from)?;
        let value: serde_json::Value = serde_json::from_slice(&response).map_err(SwapperError::from)?;
        // Check error message
        if value.is_object()
            && let Some(message) = value["message"].as_str() {
                return Err(SwapperError::ComputeQuoteError(message.to_string()));
            }
        let quotes = serde_json::from_value(value).map_err(SwapperError::from)?;
        Ok(quotes)
    }

    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<SwapTxResponse, SwapperError> {
        let url = format!("{CHAINFLIP_API_URL}/v2/swap/{tx_hash}");
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await.map_err(SwapperError::from)?;
        let status = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(status)
    }
}
