use super::AlienTarget;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Vec<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0",
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub result: T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    pub id: u64,
    pub error: JsonRpcError,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    Value(JsonRpcResponse<T>),
    Error(JsonRpcErrorResponse),
}

pub fn batch_into_target(requests: &[JsonRpcRequest], endpoint: &str) -> AlienTarget {
    let headers = HashMap::from([("Content-Type".into(), "application/json".into())]);
    let bytes = serde_json::to_vec(requests).unwrap();
    AlienTarget {
        url: endpoint.into(),
        method: "POST".into(),
        headers: Some(headers),
        body: Some(bytes),
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn test_batch_into_target() {
        let requests = vec![
            JsonRpcRequest::new(1, "eth_gasPrice", vec![]),
            JsonRpcRequest::new(2, "eth_blockNumber", vec!["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5".into(), "latest".into()]),
            JsonRpcRequest::new(3, "eth_chainId", vec![]),
        ];
        let endpoint = "http://localhost:8080";
        let target = batch_into_target(&requests, endpoint);

        assert_eq!(target.url, endpoint);
        assert_eq!(target.method, "POST");
        assert_eq!(target.headers.unwrap().get("Content-Type").unwrap(), "application/json");
        assert_eq!(
            String::from_utf8(target.body.unwrap()).unwrap(),
            r#"[{"jsonrpc":"2.0","id":1,"method":"eth_gasPrice","params":[]},{"jsonrpc":"2.0","id":2,"method":"eth_blockNumber","params":["0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5","latest"]},{"jsonrpc":"2.0","id":3,"method":"eth_chainId","params":[]}]"#
        );
    }

    #[test]
    fn test_decode_json_rpc_error_response() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 3,
                "message": "execution reverted: revert: toAddress_outOfBounds"
            }
        }"#;
        let result = serde_json::from_str::<JsonRpcResult<String>>(json).unwrap();
        if let JsonRpcResult::Error(value) = result {
            assert_eq!(value.id, 1);
            assert_eq!(value.error.code, 3);
            assert_eq!(value.error.message, "execution reverted: revert: toAddress_outOfBounds");
        } else {
            panic!("unexpected response: {:?}", result);
        }
    }

    #[test]
    fn test_decode_json_rpc_response() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x21e3bb1a6"
        }"#;
        let result = serde_json::from_str::<JsonRpcResult<String>>(json).unwrap();
        if let JsonRpcResult::Value(value) = result {
            assert_eq!(value.id, 1);
            assert_eq!(value.result, "0x21e3bb1a6");
        } else {
            panic!("unexpected response: {:?}", result);
        }
    }
}
