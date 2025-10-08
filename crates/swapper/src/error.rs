use crate::alien::AlienError;
use gem_client::ClientError;
use gem_jsonrpc::types::JsonRpcError;
use std::fmt::Debug;

#[derive(Debug)]
pub enum SwapperError {
    NotSupportedChain,
    NotSupportedAsset,
    NotSupportedPair,
    NoAvailableProvider,
    InvalidAddress(String),
    InvalidAmount(String),
    InputAmountTooSmall,
    InvalidRoute,
    NetworkError(String),
    ABIError(String),
    ComputeQuoteError(String),
    TransactionError(String),
    NoQuoteAvailable,
    NotImplemented,
}

impl std::fmt::Display for SwapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSupportedChain => write!(f, "Not supported chain"),
            Self::NotSupportedAsset => write!(f, "Not supported asset"),
            Self::NotSupportedPair => write!(f, "Not supported pair"),
            Self::NoAvailableProvider => write!(f, "No available provider"),
            Self::InvalidAddress(addr) => write!(f, "Invalid address {}", addr),
            Self::InvalidAmount(amount) => write!(f, "Invalid amount {}", amount),
            Self::InputAmountTooSmall => write!(f, "Input amount is too small"),
            Self::InvalidRoute => write!(f, "Invalid route or route data"),
            Self::NetworkError(msg) => write!(f, "Network related error: {}", msg),
            Self::ABIError(msg) => write!(f, "ABI error: {}", msg),
            Self::ComputeQuoteError(msg) => write!(f, "Compute quote error: {}", msg),
            Self::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            Self::NoQuoteAvailable => write!(f, "No quote available"),
            Self::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl std::error::Error for SwapperError {}

impl From<AlienError> for SwapperError {
    fn from(err: AlienError) -> Self {
        match err {
            AlienError::RequestError { msg } => Self::NetworkError(format!("Alien request error: {msg}")),
            AlienError::ResponseError { msg } => Self::NetworkError(format!("Alien response error: {msg}")),
            AlienError::SigningError { msg } => Self::NetworkError(format!("Alien signing error: {msg}")),
        }
    }
}

impl From<JsonRpcError> for SwapperError {
    fn from(err: JsonRpcError) -> Self {
        Self::NetworkError(format!("JSON RPC error: {err}"))
    }
}

impl From<ClientError> for SwapperError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Network(msg) => Self::NetworkError(msg),
            ClientError::Timeout => Self::NetworkError("Request timed out".into()),
            ClientError::Http { status, len } => Self::NetworkError(format!("HTTP error: status {}, body size: {}", status, len)),
            ClientError::Serialization(msg) => Self::NetworkError(msg),
        }
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
        Self::NetworkError(format!("serde_json::Error: {err}"))
    }
}

impl From<serde_urlencoded::ser::Error> for SwapperError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::NetworkError(format!("Request query error: {err}"))
    }
}

impl From<alloy_sol_types::Error> for SwapperError {
    fn from(err: alloy_sol_types::Error) -> Self {
        Self::ABIError(format!("AlloyError: {err}"))
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
