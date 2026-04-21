use std::error::Error;
use std::fmt::{self, Formatter};

use swapper::SwapperError;

use crate::gateway::GatewayError;

#[derive(Debug, Clone)]
pub enum ResolverError {
    NetworkError(String),
    PlatformError(String),
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) | Self::PlatformError(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for ResolverError {}

impl From<SwapperError> for ResolverError {
    fn from(err: SwapperError) -> Self {
        match err {
            SwapperError::ComputeQuoteError(msg) | SwapperError::TransactionError(msg) => Self::NetworkError(msg),
            other => Self::PlatformError(other.to_string()),
        }
    }
}

impl From<GatewayError> for ResolverError {
    fn from(err: GatewayError) -> Self {
        match err {
            GatewayError::NetworkError { msg } => Self::NetworkError(msg),
            GatewayError::PlatformError { msg } => Self::PlatformError(msg),
        }
    }
}
