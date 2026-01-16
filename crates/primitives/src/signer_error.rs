use crate::HexError;

#[derive(Debug, Clone)]
pub enum SignerError {
    InvalidInput(String),
    AddressError(String),
    UnsupportedOperation(String),
}

impl std::fmt::Display for SignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SignerError::AddressError(msg) => write!(f, "Invalid address: {}", msg),
            SignerError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

impl std::error::Error for SignerError {}

impl From<serde_json::Error> for SignerError {
    fn from(error: serde_json::Error) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<HexError> for SignerError {
    fn from(error: HexError) -> Self {
        SignerError::AddressError(error.to_string())
    }
}
