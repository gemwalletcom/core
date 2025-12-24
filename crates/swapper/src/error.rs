use crate::alien::AlienError;
use gem_client::ClientError;
use gem_jsonrpc::types::JsonRpcError;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "message")]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum SwapperError {
    NotSupportedChain,
    NotSupportedAsset,
    NoAvailableProvider,
    InputAmountError { min_amount: Option<String> },
    InvalidRoute,
    ComputeQuoteError(String),
    TransactionError(String),
    NoQuoteAvailable,
}

impl std::fmt::Display for SwapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSupportedAsset => write!(f, "Not supported asset"),
            Self::NotSupportedChain => write!(f, "Not supported chain"),
            Self::NoAvailableProvider => write!(f, "No available provider"),
            Self::InputAmountError { min_amount } => {
                if let Some(min) = min_amount {
                    write!(f, "Input amount is too small (minimum {min})")
                } else {
                    write!(f, "Input amount is too small")
                }
            }
            Self::InvalidRoute => write!(f, "Invalid route or route data"),
            Self::ComputeQuoteError(msg) => write!(f, "Compute quote error: {}", msg),
            Self::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            Self::NoQuoteAvailable => write!(f, "No quote available"),
        }
    }
}

impl std::error::Error for SwapperError {}

impl SwapperError {
    /// Standard messages for legacy invalid input errors that now map to compute/transaction failures.
    pub const INVALID_AMOUNT_MESSAGE: &'static str = "Invalid amount";
    pub const INVALID_ADDRESS_MESSAGE: &'static str = "Invalid address";

    pub fn into_transaction_error(self) -> Self {
        match self {
            SwapperError::ComputeQuoteError(msg) => SwapperError::TransactionError(msg),
            other => other,
        }
    }
}

impl From<AlienError> for SwapperError {
    fn from(err: AlienError) -> Self {
        match err {
            AlienError::RequestError { msg } => Self::ComputeQuoteError(msg),
            AlienError::ResponseError { msg } => Self::ComputeQuoteError(msg),
            AlienError::Http { status, .. } => Self::ComputeQuoteError(format!("HTTP error: status {}", status)),
        }
    }
}

impl From<JsonRpcError> for SwapperError {
    fn from(err: JsonRpcError) -> Self {
        Self::ComputeQuoteError(format!("JSON RPC error: {err}"))
    }
}

impl From<ClientError> for SwapperError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::Network(msg) => Self::ComputeQuoteError(msg),
            ClientError::Timeout => Self::ComputeQuoteError("Request timed out".into()),
            ClientError::Http { status, len } => Self::ComputeQuoteError(format!("HTTP error: status {}, body size: {}", status, len)),
            ClientError::Serialization(msg) => Self::ComputeQuoteError(msg),
        }
    }
}

impl From<alloy_primitives::AddressError> for SwapperError {
    fn from(err: alloy_primitives::AddressError) -> Self {
        Self::ComputeQuoteError(format!("Invalid address: {err}"))
    }
}

impl From<sui_types::AddressParseError> for SwapperError {
    fn from(err: sui_types::AddressParseError) -> Self {
        Self::ComputeQuoteError(format!("Invalid address: {err}"))
    }
}

impl From<serde_json::Error> for SwapperError {
    fn from(err: serde_json::Error) -> Self {
        Self::ComputeQuoteError(format!("serde_json::Error: {err}"))
    }
}

impl From<serde_urlencoded::ser::Error> for SwapperError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        Self::ComputeQuoteError(format!("Request query error: {err}"))
    }
}

impl From<alloy_sol_types::Error> for SwapperError {
    fn from(err: alloy_sol_types::Error) -> Self {
        Self::ComputeQuoteError(format!("AlloyError: {err}"))
    }
}

impl From<alloy_primitives::ruint::ParseError> for SwapperError {
    fn from(err: alloy_primitives::ruint::ParseError) -> Self {
        Self::ComputeQuoteError(format!("Invalid amount: {err}"))
    }
}

impl From<std::num::ParseIntError> for SwapperError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ComputeQuoteError(format!("Invalid amount: {err}"))
    }
}

impl From<num_bigint::ParseBigIntError> for SwapperError {
    fn from(err: num_bigint::ParseBigIntError) -> Self {
        Self::ComputeQuoteError(format!("Invalid amount: {err}"))
    }
}

impl From<number_formatter::NumberFormatterError> for SwapperError {
    fn from(err: number_formatter::NumberFormatterError) -> Self {
        Self::ComputeQuoteError(format!("Invalid amount: {err}"))
    }
}
