use std::error::Error;
use std::fmt::{self, Display, Formatter};

use alloy_primitives::hex::FromHexError;
use alloy_primitives::ruint::ParseError;

#[derive(Debug, Clone)]
pub enum YielderError {
    NetworkError(String),
    NotSupportedChain,
    NotSupportedAsset,
}

impl fmt::Display for YielderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "{msg}"),
            Self::NotSupportedChain => write!(f, "Not supported chain"),
            Self::NotSupportedAsset => write!(f, "Not supported asset"),
        }
    }
}

impl Error for YielderError {}

impl From<FromHexError> for YielderError {
    fn from(err: FromHexError) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<ParseError> for YielderError {
    fn from(err: ParseError) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for YielderError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::NetworkError(err.to_string())
    }
}