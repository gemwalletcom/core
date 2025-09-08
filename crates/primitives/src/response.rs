use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResult<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl<T> ResponseResult<T> {
    pub fn new(data: T) -> Self {
        Self { data: Some(data), error: None }
    }
}

impl<T: Default> ResponseResult<T> {
    pub fn error(error: String) -> Self {
        Self {
            data: None,
            error: Some(ResponseError { message: error }),
        }
    }
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub message: String,
}
