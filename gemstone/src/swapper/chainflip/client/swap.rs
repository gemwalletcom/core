use super::{
    SwapTxResponse,
    model::{QuoteRequest, QuoteResponse},
};
use crate::swapper::SwapperError;
use gem_client::{Client, ClientError};
use serde_json::Value;
use serde_urlencoded;
use std::fmt::Debug;

const QUOTE_PATH: &str = "/v2/quote";
const SWAP_PATH: &str = "/v2/swap";

#[derive(Clone, Debug)]
pub struct ChainflipClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> ChainflipClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Vec<QuoteResponse>, SwapperError> {
        let query = serde_urlencoded::to_string(request).map_err(SwapperError::from)?;
        let path = format!("{QUOTE_PATH}?{query}");
        let value: Value = self.client.get(&path).await.map_err(map_client_error)?;

        if let Some(message) = value.get("message").and_then(Value::as_str) {
            return Err(SwapperError::ComputeQuoteError(message.to_string()));
        }

        let quotes = serde_json::from_value(value).map_err(SwapperError::from)?;
        Ok(quotes)
    }

    pub async fn get_tx_status(&self, tx_hash: &str) -> Result<SwapTxResponse, SwapperError> {
        let path = format!("{SWAP_PATH}/{tx_hash}");
        self.client.get(&path).await.map_err(map_client_error)
    }
}

fn map_client_error(err: ClientError) -> SwapperError {
    SwapperError::from(err)
}
