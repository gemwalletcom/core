use crate::network::{AlienError, JsonRpcError};
use std::fmt::Debug;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Not supported asset")]
    NotSupportedAsset,
    #[error("Not supported pair")]
    NotSupportedPair,
    #[error("No available provider")]
    NoAvailableProvider,
    #[error("Invalid address {0}")]
    InvalidAddress(String),
    #[error("Invalid amount {0}")]
    InvalidAmount(String),
    #[error("Input amount is too small")]
    InputAmountTooSmall,
    #[error("Invalid route or route data")]
    InvalidRoute,
    #[error("Network related error: {0}")]
    NetworkError(String),
    #[error("ABI error: {0}")]
    ABIError(String),
    #[error("Compute quote error: {0}")]
    ComputeQuoteError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("No quote available")]
    NoQuoteAvailable,
    #[error("Not implemented")]
    NotImplemented,
}

impl From<AlienError> for SwapperError {
    fn from(err: AlienError) -> Self {
        match err {
            AlienError::RequestError { msg } => Self::NetworkError(format!("Alien request error: {}", msg)),
            AlienError::ResponseError { msg } => Self::NetworkError(format!("Alien response error: {}", msg)),
        }
    }
}

impl From<JsonRpcError> for SwapperError {
    fn from(err: JsonRpcError) -> Self {
        Self::NetworkError(format!("JSON RPC error: {}", err))
    }
}

impl From<alloy_primitives::AddressError> for SwapperError {
    fn from(err: alloy_primitives::AddressError) -> Self {
        Self::InvalidAddress(err.to_string())
    }
}

impl From<sui_types::AddressParseError> for SwapperError {
    fn from(err: sui_types::AddressParseError) -> Self {
        Self::InvalidAddress(err.to_string())
    }
}

impl From<serde_json::Error> for SwapperError {
    fn from(err: serde_json::Error) -> Self {
        Self::NetworkError(format!("serde_json::Error: {}", err))
    }
}

impl From<serde_urlencoded::ser::Error> for SwapperError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::NetworkError(format!("Request query error: {}", err))
    }
}

impl From<alloy_sol_types::Error> for SwapperError {
    fn from(err: alloy_sol_types::Error) -> Self {
        Self::ABIError(format!("AlloyError: {}", err))
    }
}

impl From<alloy_primitives::ruint::ParseError> for SwapperError {
    fn from(err: alloy_primitives::ruint::ParseError) -> Self {
        Self::InvalidAmount(err.to_string())
    }
}

impl From<std::num::ParseIntError> for SwapperError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::InvalidAmount(err.to_string())
    }
}

impl From<num_bigint::ParseBigIntError> for SwapperError {
    fn from(err: num_bigint::ParseBigIntError) -> Self {
        Self::InvalidAmount(err.to_string())
    }
}
