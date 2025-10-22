use gem_client::ClientError;
use gem_jsonrpc::RpcClientError;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    Http { status: u16, len: u64 },
}

impl AlienError {
    pub fn request_error(msg: impl Into<String>) -> Self {
        Self::RequestError { msg: msg.into() }
    }

    pub fn response_error(msg: impl Into<String>) -> Self {
        Self::ResponseError { msg: msg.into() }
    }

    pub fn http_error(status: u16, len: usize) -> Self {
        Self::Http { status, len: len as u64 }
    }
}

impl std::fmt::Display for AlienError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError { msg } => write!(f, "Request error: {}", msg),
            Self::ResponseError { msg } => write!(f, "Response error: {}", msg),
            Self::Http { status, len } => write!(f, "HTTP error: status {}, body len: {}", status, len),
        }
    }
}

impl std::error::Error for AlienError {}

impl RpcClientError for AlienError {
    fn into_client_error(self) -> ClientError {
        match self {
            Self::RequestError { msg } => ClientError::Network(format!("Alien request error: {msg}")),
            Self::ResponseError { msg } => ClientError::Network(format!("Alien response error: {msg}")),
            Self::Http { status, len } => {
                let len = usize::try_from(len).unwrap_or(usize::MAX);
                ClientError::Http { status, len }
            }
        }
    }
}
