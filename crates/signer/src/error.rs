#[derive(Debug)]
pub struct SignerError {
    pub(crate) message: String,
}

impl SignerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
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
