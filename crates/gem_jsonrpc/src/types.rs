use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};
use thiserror::Error;

pub const JSONRPC_VERSION: &str = "2.0";

pub const ERROR_INVALID_REQUEST: i32 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
pub const ERROR_INVALID_PARAMS: i32 = -32602;
pub const ERROR_INTERNAL_ERROR: i32 = -32603;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

pub trait JsonRpcRequestConvert {
    fn to_req(&self, id: u64) -> JsonRpcRequest;
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Error)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub result: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    pub id: Option<u64>,
    pub error: JsonRpcError,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    Value(JsonRpcResponse<T>),
    Error(JsonRpcErrorResponse),
}

impl<T> JsonRpcResult<T> {
    pub fn take(self) -> Result<T, JsonRpcError> {
        match self {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }
}

pub struct JsonRpcResults<T>(pub Vec<JsonRpcResult<T>>);

impl<T> JsonRpcResults<T> {
    pub fn extract(self) -> Vec<T> {
        let mut extracted = Vec::new();
        for (i, result) in self.0.into_iter().enumerate() {
            match result {
                JsonRpcResult::Value(response) => {
                    extracted.push(response.result);
                }
                JsonRpcResult::Error(error) => {
                    eprintln!("Batch call error for request {}: {:?}", i, error);
                    // Continue processing other results
                }
            }
        }
        extracted
    }
}

impl<T> Default for JsonRpcResults<T> {
    fn default() -> Self {
        JsonRpcResults(Vec::new())
    }
}

impl<T> From<Vec<JsonRpcResult<T>>> for JsonRpcResults<T> {
    fn from(vec: Vec<JsonRpcResult<T>>) -> Self {
        JsonRpcResults(vec)
    }
}

impl<T> IntoIterator for JsonRpcResults<T> {
    type Item = JsonRpcResult<T>;
    type IntoIter = std::vec::IntoIter<JsonRpcResult<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
