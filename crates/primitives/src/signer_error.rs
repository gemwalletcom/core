use crate::HexError;

#[derive(Debug, Clone)]
pub enum SignerError {
    InvalidInput(String),
    SigningError(String),
}

impl std::fmt::Display for SignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SignerError::SigningError(msg) => write!(f, "Signing error: {}", msg),
        }
    }
}

impl std::error::Error for SignerError {}

impl SignerError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn invalid_input_err<T>(message: impl Into<String>) -> Result<T, Self> {
        Err(Self::invalid_input(message))
    }

    pub fn signing_error(message: impl Into<String>) -> Self {
        Self::SigningError(message.into())
    }

    pub fn from_display(e: impl std::fmt::Display) -> Self {
        Self::InvalidInput(e.to_string())
    }
}

impl From<serde_json::Error> for SignerError {
    fn from(error: serde_json::Error) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<HexError> for SignerError {
    fn from(error: HexError) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<&str> for SignerError {
    fn from(error: &str) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SignerError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<num_bigint::ParseBigIntError> for SignerError {
    fn from(error: num_bigint::ParseBigIntError) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}
