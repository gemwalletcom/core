use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult<T> {
    Success(T),
    Error(ResponseError),
}

impl<T> ResponseResult<T> {
    pub fn new(data: T) -> Self {
        ResponseResult::Success(data)
    }

    pub fn error(error: String) -> Self {
        ResponseResult::Error(ResponseError {
            error: ErrorDetail { message: error, data: None },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResultNew<T> {
    pub data: T,
}

impl<T> ResponseResultNew<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}
