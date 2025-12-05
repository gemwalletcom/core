use crate::SwapperError;
use gem_client::{Client, build_path_with_query};
use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

const API_VERSION: u8 = 1;

#[derive(Debug, Deserialize)]
enum ProxyResult<T> {
    Ok { ok: T },
    Err { err: ProxyError },
}

#[derive(Debug, Deserialize)]
enum ProxyError {
    Object { code: SwapperError },
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
        let path = build_path_with_query("/quote", &VersionQuery { v: API_VERSION }).map_err(SwapperError::from)?;
        let response: ProxyResult<ProxyQuote> = self.client.post(&path, &request, None).await.map_err(SwapperError::from)?;
        match response {
            ProxyResult::Ok { ok } => Ok(ok),
            ProxyResult::Err { err } => Err(map_proxy_error(err)),
        }
    }

    pub async fn get_quote_data(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        let path = build_path_with_query("/quote_data", &VersionQuery { v: API_VERSION }).map_err(SwapperError::from)?;
        let response: ProxyResult<SwapQuoteData> = self.client.post(&path, &quote, None).await.map_err(SwapperError::from)?;
        match response {
            ProxyResult::Ok { ok } => Ok(ok),
            ProxyResult::Err { err } => Err(map_proxy_error(err)),
        }
    }
}

#[derive(Debug, Serialize)]
struct VersionQuery {
    v: u8,
}

/// Try to cast a proxy error back into a `SwapperError` variant.
fn map_proxy_error(error: ProxyError) -> SwapperError {
    match error {
        ProxyError::Object { code } => code,
    }
}
