use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResult<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl<T> ResponseResult<T> {
    pub fn new(data: T) -> Self {
        Self { data, error: None }
    }
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub error: String,
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ResponseError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self { error: error.to_string() }
    }
}
