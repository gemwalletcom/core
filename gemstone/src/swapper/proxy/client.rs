use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use primitives::swap::{Quote, QuoteData, QuoteRequest};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProxyResult<T> {
    Ok(T),
    Err { error: String },
}

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

        match serde_json::from_slice::<ProxyResult<Quote>>(&data).map_err(SwapperError::from)? {
            ProxyResult::Ok(q) => Ok(q),
            ProxyResult::Err { error } => Err(SwapperError::ComputeQuoteError(error)),
        }
    }

    pub async fn get_quote_data(&self, endpoint: &str, quote: Quote) -> Result<QuoteData, SwapperError> {
        let url = format!("{}/quote_data", endpoint);
        let target = AlienTarget::post_json(&url, serde_json::json!(quote));

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        match serde_json::from_slice::<ProxyResult<QuoteData>>(&data).map_err(SwapperError::from)? {
            ProxyResult::Ok(qd) => Ok(qd),
            ProxyResult::Err { error } => Err(SwapperError::TransactionError(error)),
        }
    }
}
