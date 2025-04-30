use super::model::{QuoteRequest, QuoteResponse};
use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use std::sync::Arc;

const CHAINFLIP_API_URL: &str = "https://chainflip-swap.chainflip.io";

#[derive(Debug)]
pub struct BrokerClient {
    provider: Arc<dyn AlienProvider>,
}

impl BrokerClient {
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
}
