use super::model::{Quote, QuoteData, QuoteRequest};
use crate::{
    network::{AlienHttpMethod, AlienProvider, AlienTarget},
    swapper::SwapperError,
};
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
        let query = serde_urlencoded::to_string(&request).unwrap();
        let url = format!("{}/quote?{}", endpoint, query);

        let target = AlienTarget::get(&url);

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })
    }

    pub async fn get_quote_data(&self, endpoint: &str, quote: Quote) -> Result<QuoteData, SwapperError> {
        let url = format!("{}/quote_data", endpoint);
        let target = AlienTarget::post_json(&url, serde_json::json!(quote));

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        serde_json::from_slice(&data).map_err(|err| SwapperError::NetworkError { msg: err.to_string() })
    }
}
