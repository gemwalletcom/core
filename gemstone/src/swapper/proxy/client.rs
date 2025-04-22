use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use primitives::swap::{Quote, QuoteData, QuoteRequest};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProxyClient {
    provider: Arc<dyn AlienProvider>,
}

impl ProxyClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_quote(&self, endpoint: &str, request: QuoteRequest) -> Result<Quote, SwapperError> {
        let url = format!("{}/quote", endpoint);
        let target = AlienTarget::post_json(&url, serde_json::json!(request));
        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        serde_json::from_slice(&data).map_err(SwapperError::from)
    }

    pub async fn get_quote_data(&self, endpoint: &str, quote: Quote) -> Result<QuoteData, SwapperError> {
        let url = format!("{}/quote_data", endpoint);
        let target = AlienTarget::post_json(&url, serde_json::json!(quote));

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        serde_json::from_slice(&data).map_err(SwapperError::from)
    }
}
