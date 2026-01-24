use gem_client::ClientError;
use gem_jsonrpc::RpcClientError;

#[derive(Debug, Clone)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    Http { status: u16, len: u32 },
}

impl AlienError {
    pub fn request_error(msg: impl Into<String>) -> Self {
        Self::RequestError { msg: msg.into() }
    }

    pub fn response_error(msg: impl Into<String>) -> Self {
        Self::ResponseError { msg: msg.into() }
    }

    pub fn http_error(status: u16, len: usize) -> Self {
        Self::Http {
            status,
            len: len.min(u32::MAX as usize) as u32,
        }
    }
}

impl std::fmt::Display for AlienError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError { msg } => write!(f, "Request error: {}", msg),
            Self::ResponseError { msg } => write!(f, "Response error: {}", msg),
            Self::Http { status, .. } => write!(f, "HTTP error: status {}", status),
        }
    }
}

impl std::error::Error for AlienError {}

impl RpcClientError for AlienError {
    fn into_client_error(self) -> ClientError {
        match self {
            Self::RequestError { msg } => ClientError::Network(msg),
            Self::ResponseError { msg } => ClientError::Network(msg),
            Self::Http { status, .. } => ClientError::Http { status, body: Vec::new() },
        }
    }
}
