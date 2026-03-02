use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};

pub const JSONRPC_VERSION: &str = "2.0";

pub const ERROR_INVALID_REQUEST: i32 = -32600;
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;
pub const ERROR_INVALID_PARAMS: i32 = -32602;
pub const ERROR_INTERNAL_ERROR: i32 = -32603;

pub const ERROR_CLIENT_ERROR: i32 = -32900;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let original = self.message.trim();
        let message = if original.is_empty() && self.code == ERROR_CLIENT_ERROR {
            "Client error"
        } else {
            original
        };

        write!(f, "{} ({})", message, self.code)
    }
}

impl std::error::Error for JsonRpcError {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    pub id: Option<u64>,
    pub result: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    pub id: Option<u64>,
    pub error: JsonRpcError,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    Value(JsonRpcResponse<T>),
    Error(JsonRpcErrorResponse),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for JsonRpcResult<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;
        let id = raw.get("id").and_then(|v| v.as_u64());

        if let Some(error) = raw.get("error") {
            let error: JsonRpcError = serde_json::from_value(error.clone()).map_err(serde::de::Error::custom)?;
            return Ok(JsonRpcResult::Error(JsonRpcErrorResponse { id, error }));
        }

        let Some(result) = raw.get("result") else {
            return Err(serde::de::Error::custom(format!("missing result and error fields, raw: {raw}")));
        };

        let result = T::deserialize(result.clone()).map_err(|e| serde::de::Error::custom(format!("failed to deserialize result: {e}, raw: {result}")))?;
        Ok(JsonRpcResult::Value(JsonRpcResponse { id, result }))
    }
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
    pub fn take_all(self) -> Result<Vec<T>, JsonRpcError> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                r.take().map_err(|e| JsonRpcError {
                    code: e.code,
                    message: format!("batch request [{}]: {}", i, e.message),
                })
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_error_display_with_client_error_code() {
        let error = JsonRpcError {
            code: ERROR_CLIENT_ERROR,
            message: "".into(),
        };

        assert_eq!(format!("{error}"), "Client error (-32900)");
    }

    #[test]
    fn test_jsonrpc_error_display_with_method_not_found_code() {
        let error = JsonRpcError {
            code: ERROR_METHOD_NOT_FOUND,
            message: "Method not found".into(),
        };

        assert_eq!(format!("{error}"), "Method not found (-32601)");
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Block {
        number: String,
    }

    #[test]
    fn test_deserialize_success() {
        let json = r#"{"id": 1, "result": {"number": "0x10"}}"#;
        let result: JsonRpcResult<Block> = serde_json::from_str(json).unwrap();
        assert!(matches!(result, JsonRpcResult::Value(r) if r.result.number == "0x10"));
    }

    #[test]
    fn test_deserialize_error_response() {
        let json = r#"{"id": 1, "error": {"code": -32601, "message": "Method not found"}}"#;
        let result: JsonRpcResult<Block> = serde_json::from_str(json).unwrap();
        assert!(matches!(result, JsonRpcResult::Error(e) if e.error.code == -32601));
    }

    #[test]
    fn test_deserialize_null_result_fails_with_detail() {
        let json = r#"{"id": 1, "result": null}"#;
        let err = serde_json::from_str::<JsonRpcResult<Block>>(json).unwrap_err();
        assert!(
            err.to_string()
                .contains("failed to deserialize result: invalid type: null, expected struct Block, raw: null")
        );
    }

    #[test]
    fn test_deserialize_null_result_ok_for_option() {
        let json = r#"{"id": 1, "result": null}"#;
        let result: JsonRpcResult<Option<Block>> = serde_json::from_str(json).unwrap();
        assert!(matches!(result, JsonRpcResult::Value(r) if r.result.is_none()));
    }

    #[test]
    fn test_deserialize_batch_with_mixed_results() {
        let json = r#"[
            {"id": 1, "result": {"number": "0x10"}},
            {"id": 2, "error": {"code": -32600, "message": "Invalid"}}
        ]"#;
        let results: Vec<JsonRpcResult<Block>> = serde_json::from_str(json).unwrap();
        assert!(matches!(&results[0], JsonRpcResult::Value(_)));
        assert!(matches!(&results[1], JsonRpcResult::Error(_)));
    }

    #[test]
    fn test_take_all_success() {
        let results: JsonRpcResults<Block> = vec![
            JsonRpcResult::Value(JsonRpcResponse { id: Some(1), result: Block { number: "0x10".into() } }),
            JsonRpcResult::Value(JsonRpcResponse { id: Some(2), result: Block { number: "0x20".into() } }),
        ]
        .into();

        let values = results.take_all().unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].number, "0x10");
        assert_eq!(values[1].number, "0x20");
    }

    #[test]
    fn test_take_all_error_includes_index() {
        let results: JsonRpcResults<Block> = vec![
            JsonRpcResult::Value(JsonRpcResponse { id: Some(1), result: Block { number: "0x10".into() } }),
            JsonRpcResult::Error(JsonRpcErrorResponse {
                id: Some(2),
                error: JsonRpcError { code: ERROR_INVALID_REQUEST, message: "Invalid".into() },
            }),
        ]
        .into();

        let err = results.take_all().unwrap_err();
        assert_eq!(err.code, ERROR_INVALID_REQUEST);
        assert_eq!(err.message, "batch request [1]: Invalid");
    }
}
