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
        let url = format!("{}/v2/quote?{}", CHAINFLIP_API_URL, query);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await.map_err(SwapperError::from)?;
        let quote = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(quote)
    }

    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<SwapTxResponse, SwapperError> {
        let url = format!("{}/v2/swap/{}", CHAINFLIP_API_URL, tx_hash);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await.map_err(SwapperError::from)?;
        let status = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(status)
    }
}
