use std::error::Error;
use std::fmt::{self, Display, Formatter};

use alloy_primitives::hex::FromHexError;
use alloy_primitives::ruint::ParseError;
use gem_client::ClientError;
use gem_jsonrpc::types::JsonRpcError;

#[derive(Debug, Clone)]
pub enum YielderError {
    InvalidAddress(String),
    InvalidAmount(String),
    UnsupportedAsset(String),
    UnsupportedChain(String),
    ProviderNotFound(String),
    NetworkError(String),
}

impl YielderError {
    pub fn unsupported_asset(asset: &impl Display) -> Self {
        Self::UnsupportedAsset(asset.to_string())
    }

    pub fn unsupported_chain(chain: &impl Display) -> Self {
        Self::UnsupportedChain(chain.to_string())
    }

    pub fn provider_not_found(provider: &impl Display) -> Self {
        Self::ProviderNotFound(provider.to_string())
    }
}

impl fmt::Display for YielderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(msg) => write!(f, "Invalid address: {msg}"),
            Self::InvalidAmount(msg) => write!(f, "Invalid amount: {msg}"),
            Self::UnsupportedAsset(msg) => write!(f, "Unsupported asset: {msg}"),
            Self::UnsupportedChain(msg) => write!(f, "Unsupported chain: {msg}"),
            Self::ProviderNotFound(msg) => write!(f, "Provider not found: {msg}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl Error for YielderError {}

impl From<FromHexError> for YielderError {
    fn from(err: FromHexError) -> Self {
        Self::InvalidAddress(err.to_string())
    }
}

impl From<ParseError> for YielderError {
    fn from(err: ParseError) -> Self {
        Self::InvalidAmount(err.to_string())
    }
}

impl From<ClientError> for YielderError {
    fn from(err: ClientError) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<JsonRpcError> for YielderError {
    fn from(err: JsonRpcError) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for YielderError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::NetworkError(err.to_string())
    }
}

impl From<strum::ParseError> for YielderError {
    fn from(err: strum::ParseError) -> Self {
        Self::ProviderNotFound(err.to_string())
    }
}
