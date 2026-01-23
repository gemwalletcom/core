use crate::SwapperError;
use gem_client::{Client, build_path_with_query};
use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

const API_VERSION: u8 = 1;

#[derive(Debug, Serialize)]
struct VersionQuery {
    v: u8,
}

#[derive(Debug, Deserialize)]
pub struct ProxyError {
    pub err: SwapperError,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProxyResponse<T> {
    Ok { ok: T },
    Err(ProxyError),
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
        let response: ProxyResponse<ProxyQuote> = self.client.post(&path, &request, None).await.map_err(SwapperError::from)?;
        match response {
            ProxyResponse::Ok { ok } => Ok(ok),
            ProxyResponse::Err(e) => Err(e.err),
        }
    }

    pub async fn get_quote_data(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        let path = build_path_with_query("/quote_data", &VersionQuery { v: API_VERSION }).map_err(SwapperError::from)?;
        let response: ProxyResponse<SwapQuoteData> = self.client.post(&path, &quote, None).await.map_err(SwapperError::from)?;
        match response {
            ProxyResponse::Ok { ok } => Ok(ok),
            ProxyResponse::Err(e) => Err(e.err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_error_deserialization() {
        let json = r#"{"err": {"type": "compute_quote_error", "message": "Amount too small (min ~0.0008099 ETH)"}}"#;
        let error: ProxyError = serde_json::from_str(json).unwrap();

        assert!(matches!(error.err, SwapperError::ComputeQuoteError(_)));

        if let SwapperError::ComputeQuoteError(msg) = error.err {
            assert!(msg.contains("Amount too small"));
        }
    }
}
