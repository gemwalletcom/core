use std::error::Error;
use std::fmt::{self, Formatter};

use crate::gateway::GatewayError;

#[derive(Debug, Clone)]
pub enum TransactionStatusError {
    NetworkError(String),
    PlatformError(String),
}

impl fmt::Display for TransactionStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) | Self::PlatformError(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for TransactionStatusError {}

impl From<GatewayError> for TransactionStatusError {
    fn from(err: GatewayError) -> Self {
        match err {
            GatewayError::NetworkError { msg } => Self::NetworkError(msg),
            GatewayError::PlatformError { msg } => Self::PlatformError(msg),
        }
    }
}
