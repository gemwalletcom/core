use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeFiError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<reqwest::Error> for DeFiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DeFiError::NetworkError("Request timeout".to_string())
        } else if err.is_connect() {
            DeFiError::NetworkError("Connection error".to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            DeFiError::RateLimitExceeded
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            DeFiError::AuthError("Unauthorized".to_string())
        } else {
            DeFiError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for DeFiError {
    fn from(err: serde_json::Error) -> Self {
        DeFiError::InvalidResponse(format!("JSON parsing error: {err}"))
    }
}
