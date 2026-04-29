use primitives::SignerError;

#[derive(Debug, Clone)]
pub struct TvmError(pub String);

impl TvmError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for TvmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TvmError {}

impl From<TvmError> for SignerError {
    fn from(err: TvmError) -> Self {
        SignerError::InvalidInput(err.0)
    }
}
