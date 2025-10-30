use alloy_primitives::{BigIntConversionError, ParseSignedError, ruint};

#[derive(Debug)]
pub struct SignerError {
    pub(crate) message: String,
}

impl SignerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(message)
    }

    pub fn invalid_input_err<T>(message: impl Into<String>) -> Result<T, Self> {
        Err(Self::invalid_input(message))
    }
}

impl std::fmt::Display for SignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SignerError {}

impl From<&str> for SignerError {
    fn from(value: &str) -> Self {
        SignerError::new(value)
    }
}

impl From<String> for SignerError {
    fn from(value: String) -> Self {
        SignerError::new(value)
    }
}

impl From<hex::FromHexError> for SignerError {
    fn from(error: hex::FromHexError) -> Self {
        SignerError::new(format!("Invalid hex string: {error}"))
    }
}

impl From<alloy_primitives::hex::FromHexError> for SignerError {
    fn from(error: alloy_primitives::hex::FromHexError) -> Self {
        SignerError::new(format!("Invalid hex string: {error}"))
    }
}

impl From<ParseSignedError> for SignerError {
    fn from(error: ParseSignedError) -> Self {
        SignerError::new(format!("Invalid signed integer: {error}"))
    }
}

impl From<ruint::ParseError> for SignerError {
    fn from(error: ruint::ParseError) -> Self {
        SignerError::new(format!("Invalid integer: {error}"))
    }
}

impl From<BigIntConversionError> for SignerError {
    fn from(error: BigIntConversionError) -> Self {
        SignerError::new(format!("Integer conversion failed: {error}"))
    }
}

impl From<serde_json::Error> for SignerError {
    fn from(error: serde_json::Error) -> Self {
        SignerError::new(format!("Invalid JSON: {error}"))
    }
}
