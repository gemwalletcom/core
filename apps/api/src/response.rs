use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResults<T> {
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub error: String,
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ResponseError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}
