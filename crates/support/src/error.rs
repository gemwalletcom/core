use reqwest::Error as ReqwestError;
use serde_json::Error as JsonError;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum ChatwootError {
    Http { status: u16, message: String },
    Request(String),
    InvalidResponse(String),
}

impl ChatwootError {
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http { status, message: message.into() }
    }

    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponse(message.into())
    }

    pub fn is_session_error(&self) -> bool {
        match self {
            Self::Http { status, .. } => *status == 401 || *status == 404,
            Self::Request(_) | Self::InvalidResponse(_) => false,
        }
    }
}

impl Display for ChatwootError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatwootError::Http { status, message } => write!(f, "Chatwoot HTTP error {status}: {message}"),
            ChatwootError::Request(message) => write!(f, "Chatwoot request error: {message}"),
            ChatwootError::InvalidResponse(message) => write!(f, "Chatwoot invalid response: {message}"),
        }
    }
}

impl Error for ChatwootError {}

impl From<ReqwestError> for ChatwootError {
    fn from(error: ReqwestError) -> Self {
        match error.status() {
            Some(status) => ChatwootError::http(status.as_u16(), error.to_string()),
            None => ChatwootError::Request(error.to_string()),
        }
    }
}

impl From<JsonError> for ChatwootError {
    fn from(error: JsonError) -> Self {
        ChatwootError::InvalidResponse(error.to_string())
    }
}
