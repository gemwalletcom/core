#[derive(Debug, Clone)]
pub enum AlienError {
    RequestError { msg: String },
    ResponseError { msg: String },
    SigningError { msg: String },
}

impl std::fmt::Display for AlienError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError { msg } => write!(f, "Request error: {}", msg),
            Self::ResponseError { msg } => write!(f, "Response error: {}", msg),
            Self::SigningError { msg } => write!(f, "Signing error: {}", msg),
        }
    }
}

impl std::error::Error for AlienError {}
