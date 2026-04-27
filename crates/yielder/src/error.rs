use std::error::Error;
use std::fmt::{self, Formatter};

use alloy_primitives::hex::FromHexError;
use alloy_primitives::ruint::ParseError;
use primitives::SignerError;

#[derive(Debug, Clone)]
pub enum YielderError {
    NetworkError(String),
    InvalidInput(String),
    NotSupportedChain,
    NotSupportedAsset,
}

impl fmt::Display for YielderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "{msg}"),
            Self::InvalidInput(msg) => write!(f, "{msg}"),
            Self::NotSupportedChain => write!(f, "Not supported chain"),
            Self::NotSupportedAsset => write!(f, "Not supported asset"),
        }
    }
}

impl Error for YielderError {}

impl YielderError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}

impl From<FromHexError> for YielderError {
    fn from(err: FromHexError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<ParseError> for YielderError {
    fn from(err: ParseError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<SignerError> for YielderError {
    fn from(err: SignerError) -> Self {
        match err {
            SignerError::InvalidInput(message) | SignerError::SigningError(message) => Self::InvalidInput(message),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for YielderError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::NetworkError(err.to_string())
    }
}
