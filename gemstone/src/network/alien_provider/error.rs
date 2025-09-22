#[derive(Debug, Clone, uniffi::Error)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
}

impl std::fmt::Display for AlienError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError { msg } => write!(f, "Request is invalid: {}", msg),
            Self::ResponseError { msg } => write!(f, "Request error: {}", msg),
        }
    }
}

impl std::error::Error for AlienError {}
