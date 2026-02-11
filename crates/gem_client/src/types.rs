use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: Option<u16>,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub enum ClientError {
    Network(String),
    Timeout,
    Http { status: u16, body: Vec<u8> },
    Serialization(String),
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => f.debug_tuple("Network").field(msg).finish(),
            Self::Timeout => write!(f, "Timeout"),
            Self::Http { status, body } => {
                let body_str = String::from_utf8_lossy(&body[..body.len().min(256)]);
                f.debug_struct("Http").field("status", status).field("body", &body_str).finish()
            }
            Self::Serialization(msg) => f.debug_tuple("Serialization").field(msg).finish(),
        }
    }
}

pub fn decode_json_byte_array(values: Vec<Value>) -> Result<Vec<u8>, ClientError> {
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        let byte = value
            .as_u64()
            .ok_or_else(|| ClientError::Serialization("Expected byte array for binary content-type".to_string()))?;
        if byte > u8::MAX as u64 {
            return Err(ClientError::Serialization("Binary body byte out of range".to_string()));
        }
        bytes.push(byte as u8);
    }
    Ok(bytes)
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Timeout => write!(f, "Timeout error"),
            Self::Http { status, .. } => write!(f, "HTTP error: status {}", status),
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

pub fn deserialize_response<R>(response: &Response) -> Result<R, ClientError>
where
    R: DeserializeOwned,
{
    match serde_json::from_slice(&response.data) {
        Ok(value) => Ok(value),
        Err(error) => {
            validate_http_status(response)?;
            Err(ClientError::Serialization(error.to_string()))
        }
    }
}

fn validate_http_status(response: &Response) -> Result<(), ClientError> {
    if let Some(status) = response.status {
        if !(200..400).contains(&status) {
            return Err(ClientError::Http {
                status,
                body: response.data.clone(),
            });
        }
    }
    Ok(())
}
