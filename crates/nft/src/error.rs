use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NFTError {
    ProviderNotFound(String),
    AssetNotFound(String),
    CollectionNotFound(String),
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for NFTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NFTError::ProviderNotFound(provider) => write!(f, "NFT provider not found: {}", provider),
            NFTError::AssetNotFound(id) => write!(f, "NFT asset not found: {}", id),
            NFTError::CollectionNotFound(id) => write!(f, "NFT collection not found: {}", id),
            NFTError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            NFTError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for NFTError {}
