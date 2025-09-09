use serde::{Deserialize, Serialize};

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
            error: ErrorDetail { message: error },
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResultOld<T> {
    pub data: T,
}

impl<T> ResponseResultOld<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}
