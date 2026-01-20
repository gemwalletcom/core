use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: Option<u16>,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ClientError {
    Network(String),
    Timeout,
    Http { status: u16, len: usize },
    Serialization(String),
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

/// Deserializes a response, trying to decode the model first.
/// If deserialization fails, checks if it's an HTTP error before returning a serde error.
pub fn deserialize_response<R>(response: &Response) -> Result<R, ClientError>
where
    R: DeserializeOwned,
{
    match serde_json::from_slice(&response.data) {
        Ok(value) => Ok(value),
        Err(error) => {
            validate_http_status(response)?;
            let preview_bytes = if response.data.len() > 256 { &response.data[..256] } else { &response.data };
            let body_preview = String::from_utf8_lossy(preview_bytes);
            Err(ClientError::Serialization(format!(
                "Failed to deserialize response: {error}. Response body: {body_preview}"
            )))
        }
    }
}

fn validate_http_status(response: &Response) -> Result<(), ClientError> {
    if let Some(status) = response.status {
        if !(200..400).contains(&status) {
            return Err(ClientError::Http { status, len: response.data.len() });
        }
    }
    Ok(())
}
