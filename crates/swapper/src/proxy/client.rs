use crate::SwapperError;
use gem_client::{Client, ClientError};
use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProxyResult<T> {
    Ok(T),
    Err { error: String },
}

#[derive(Clone, Debug)]
pub struct ProxyClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> ProxyClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: ProxyQuoteRequest) -> Result<ProxyQuote, SwapperError> {
        let response: ProxyResult<ProxyQuote> = self.client.post("/quote", &request, None).await.map_err(map_client_error)?;
        match response {
            ProxyResult::Ok(q) => Ok(q),
            ProxyResult::Err { error } => Err(SwapperError::ComputeQuoteError(error)),
        }
    }

    pub async fn get_quote_data(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        let response: ProxyResult<SwapQuoteData> = self.client.post("/quote_data", &quote, None).await.map_err(map_client_error)?;
        match response {
            ProxyResult::Ok(qd) => Ok(qd),
            ProxyResult::Err { error } => Err(SwapperError::TransactionError(error)),
        }
    }
}

fn map_client_error(err: ClientError) -> SwapperError {
    SwapperError::from(err)
}
