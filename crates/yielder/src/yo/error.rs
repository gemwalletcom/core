use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct YieldError(String);

impl YieldError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for YieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for YieldError {}

impl From<&str> for YieldError {
    fn from(value: &str) -> Self {
        YieldError::new(value)
    }
}

impl From<String> for YieldError {
    fn from(value: String) -> Self {
        YieldError::new(value)
    }
}
