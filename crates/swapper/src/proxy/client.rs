use crate::SwapperError;
use gem_client::{Client, ClientExt};
use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
pub struct ProxyError {
    pub err: SwapperError,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProxyResponse<T> {
    Ok { ok: T },
    Err { err: SwapperError },
}

impl<T> From<Result<T, SwapperError>> for ProxyResponse<T> {
    fn from(result: Result<T, SwapperError>) -> Self {
        match result {
            Ok(ok) => Self::Ok { ok },
            Err(err) => Self::Err { err },
        }
    }
}

impl<T> From<ProxyResponse<T>> for Result<T, SwapperError> {
    fn from(response: ProxyResponse<T>) -> Self {
        match response {
            ProxyResponse::Ok { ok } => Ok(ok),
            ProxyResponse::Err { err } => Err(err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProxyClient<C: Client + Clone + Debug> {
    client: C,
}

impl<C: Client + Clone + Debug> ProxyClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, request: ProxyQuoteRequest) -> Result<ProxyQuote, SwapperError> {
        self.post("/quote", &request).await
    }

    pub async fn get_quote_data(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        self.post("/quote_data", &quote).await
    }

    async fn post<Req: Serialize + Send + Sync, Res: DeserializeOwned + Send>(&self, path: &str, body: &Req) -> Result<Res, SwapperError> {
        let response: ProxyResponse<Res> = self.client.post(path, body).await?;
        response.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_error_deserialization() {
        let json = r#"{"err": {"type": "compute_quote_error", "message": "Quote failed"}}"#;
        assert_eq!(
            serde_json::from_str::<ProxyError>(json).unwrap().err,
            SwapperError::ComputeQuoteError("Quote failed".into())
        );

        let json = r#"{"err": {"type": "input_amount_error", "message": {"min_amount": "19620000"}}}"#;
        assert_eq!(
            serde_json::from_str::<ProxyError>(json).unwrap().err,
            SwapperError::InputAmountError {
                min_amount: Some("19620000".into())
            }
        );

        let json = r#"{"err": {"type": "input_amount_error", "message": {"min_amount": null}}}"#;
        assert_eq!(serde_json::from_str::<ProxyError>(json).unwrap().err, SwapperError::InputAmountError { min_amount: None });

        let json = r#"{"err": {"type": "no_quote_available"}}"#;
        assert_eq!(serde_json::from_str::<ProxyError>(json).unwrap().err, SwapperError::NoQuoteAvailable);

        let json = r#"{"err": {"type": "transaction_error", "message": "tx failed"}}"#;
        assert_eq!(serde_json::from_str::<ProxyError>(json).unwrap().err, SwapperError::TransactionError("tx failed".into()));
    }

    #[test]
    fn test_swapper_error_serialization() {
        assert_eq!(
            serde_json::to_string(&SwapperError::InputAmountError { min_amount: Some("100".into()) }).unwrap(),
            r#"{"type":"input_amount_error","message":{"min_amount":"100"}}"#
        );
        assert_eq!(
            serde_json::to_string(&SwapperError::ComputeQuoteError("error".into())).unwrap(),
            r#"{"type":"compute_quote_error","message":"error"}"#
        );
        assert_eq!(serde_json::to_string(&SwapperError::NoQuoteAvailable).unwrap(), r#"{"type":"no_quote_available"}"#);
    }
}
