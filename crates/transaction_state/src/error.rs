use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum StateError {
    NetworkError(String),
    PlatformError(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) | Self::PlatformError(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for StateError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_error_display() {
        assert_eq!(StateError::NetworkError("down".into()).to_string(), "down");
        assert_eq!(StateError::PlatformError("bad".into()).to_string(), "bad");
    }
}
