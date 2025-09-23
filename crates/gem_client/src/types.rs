use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    Network(String),
    Timeout,
    Http { status: u16, body: String },
    Serialization(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Timeout => write!(f, "Timeout error"),
            Self::Http { status, body } => write!(f, "HTTP error: status {}, body: {}", status, body),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::Serialization(format!("JSON error: {err}"))
    }
}
