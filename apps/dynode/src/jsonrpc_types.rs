use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcCall {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

impl JsonRpcCall {
    pub fn cache_key(&self, host: &str, path: &str) -> String {
        let base = format!("{}:POST:{}:{}", host, path, self.method);

        if self.params.is_null() {
            base
        } else {
            let params_str = serde_json::to_string(&self.params).unwrap_or_else(|_| self.params.to_string());
            format!("{}:{}", base, params_str)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub result: Value,
    pub id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    pub error: JsonRpcError,
    pub id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResult {
    Success(JsonRpcResponse),
    Error(JsonRpcErrorResponse),
}

impl JsonRpcResult {
    pub fn id(&self) -> u64 {
        match self {
            JsonRpcResult::Success(success) => success.id,
            JsonRpcResult::Error(error) => error.id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequestType {
    Regular { path: String, method: String, body: Vec<u8> },
    JsonRpc(JsonRpcRequest),
}

#[derive(Debug, Clone)]
pub enum JsonRpcRequest {
    Single(JsonRpcCall),
    Batch(Vec<JsonRpcCall>),
}

impl JsonRpcRequest {
    pub fn cache_key(&self, host: &str, path: &str) -> String {
        match self {
            Self::Single(call) => call.cache_key(host, path),
            Self::Batch(calls) => {
                let sorted_keys = {
                    let mut keys: Vec<String> = calls.iter().map(|call| call.cache_key(host, path)).collect();
                    keys.sort();
                    keys
                };
                format!("batch:{}", sorted_keys.join(";"))
            }
        }
    }

    pub fn get_calls(&self) -> Vec<&JsonRpcCall> {
        match self {
            Self::Single(call) => vec![call],
            Self::Batch(calls) => calls.iter().collect(),
        }
    }
}

impl RequestType {
    pub fn from_request(method: &str, path: String, body: Vec<u8>) -> Self {
        if method == "POST"
            && let Ok(body_str) = std::str::from_utf8(&body)
        {
            if let Ok(call) = serde_json::from_str::<JsonRpcCall>(body_str) {
                return RequestType::JsonRpc(JsonRpcRequest::Single(call));
            }
            if let Ok(calls) = serde_json::from_str::<Vec<JsonRpcCall>>(body_str)
                && !calls.is_empty()
            {
                return RequestType::JsonRpc(JsonRpcRequest::Batch(calls));
            }
        }
        RequestType::Regular {
            path,
            method: method.to_string(),
            body,
        }
    }

    pub fn get_methods_for_metrics(&self) -> Vec<String> {
        match self {
            Self::JsonRpc(JsonRpcRequest::Single(call)) => vec![call.method.clone()],
            Self::JsonRpc(JsonRpcRequest::Batch(calls)) => calls.iter().map(|call| call.method.clone()).collect(),
            Self::Regular { path, .. } => vec![path.clone()],
        }
    }

    pub fn get_methods_list(&self) -> String {
        self.get_methods_for_metrics().join(",")
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::JsonRpc(_) => "application/json",
            Self::Regular { .. } => "application/json",
        }
    }

    pub fn cache_key(&self, host: &str, path: &str) -> String {
        match self {
            Self::Regular { path, method, .. } => format!("{}:{}:{}", host, method, path),
            Self::JsonRpc(json_rpc) => json_rpc.cache_key(host, path),
        }
    }

    pub fn create_cache_key(host: &str, path: &str, call: &JsonRpcCall) -> String {
        let mut key = format!("{}:POST:{}", host, path);
        key.push(':');
        key.push_str(&call.method);

        if !call.params.is_null() {
            key.push(':');
            let params_str = serde_json::to_string(&call.params).unwrap_or_else(|_| call.params.to_string());
            key.push_str(&params_str);
        }

        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_key_generation() {
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: json!([]),
            id: 1,
        };

        let request = JsonRpcRequest::Single(call);
        let key = request.cache_key("example.com", "/rpc");

        assert_eq!(key, "example.com:POST:/rpc:eth_blockNumber:[]");
    }

    #[test]
    fn test_cache_key_with_params() {
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_getBalance".to_string(),
            params: json!(["0x123", "latest"]),
            id: 1,
        };

        let request = JsonRpcRequest::Single(call);
        let key = request.cache_key("example.com", "/rpc");

        assert!(key.contains("eth_getBalance"));
        assert!(key.contains("0x123"));
        assert!(key.contains("latest"));
    }

    #[test]
    fn test_cache_key_null_params() {
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: json!(null),
            id: 1,
        };

        let request = JsonRpcRequest::Single(call);
        let key = request.cache_key("example.com", "/rpc");

        assert_eq!(key, "example.com:POST:/rpc:eth_blockNumber");
    }

    #[test]
    fn test_batch_request_parsing() {
        let batch_json = r#"[
            {"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1},
            {"jsonrpc":"2.0","method":"eth_getBalance","params":["0x123","latest"],"id":2}
        ]"#;

        let body = batch_json.as_bytes().to_vec();
        let request_type = RequestType::from_request("POST", "/rpc".to_string(), body);

        match request_type {
            RequestType::JsonRpc(JsonRpcRequest::Batch(calls)) => {
                assert_eq!(calls.len(), 2);
                assert_eq!(calls[0].method, "eth_blockNumber");
                assert_eq!(calls[0].id, 1);
                assert_eq!(calls[1].method, "eth_getBalance");
                assert_eq!(calls[1].id, 2);
            }
            _ => panic!("Expected batch request"),
        }
    }

    #[test]
    fn test_batch_cache_key_generation() {
        let calls = vec![
            JsonRpcCall {
                jsonrpc: "2.0".to_string(),
                method: "eth_blockNumber".to_string(),
                params: json!([]),
                id: 1,
            },
            JsonRpcCall {
                jsonrpc: "2.0".to_string(),
                method: "eth_getBalance".to_string(),
                params: json!(["0x123", "latest"]),
                id: 2,
            },
        ];

        let request = JsonRpcRequest::Batch(calls);
        let key = request.cache_key("example.com", "/rpc");

        assert!(key.starts_with("batch:"));
        assert!(key.contains("eth_blockNumber"));
        assert!(key.contains("eth_getBalance"));
    }

    #[test]
    fn test_jsonrpc_call_cache_key() {
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_getBalance".to_string(),
            params: json!(["0x123", "latest"]),
            id: 1,
        };

        let key = call.cache_key("example.com", "/rpc");
        assert_eq!(key, "example.com:POST:/rpc:eth_getBalance:[\"0x123\",\"latest\"]");
    }

    #[test]
    fn test_jsonrpc_result_id_extraction() {
        let success = JsonRpcResult::Success(JsonRpcResponse {
            result: json!({"test": "value"}),
            id: 123,
        });

        let error = JsonRpcResult::Error(JsonRpcErrorResponse {
            error: JsonRpcError {
                code: -32602,
                message: "Invalid params".to_string(),
                data: None,
            },
            id: 456,
        });

        assert_eq!(success.id(), 123);
        assert_eq!(error.id(), 456);
    }

    #[test]
    fn test_batch_with_duplicate_ids() {
        let batch_json = r#"[
            {"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1},
            {"jsonrpc":"2.0","method":"eth_getBalance","params":["0x123","latest"],"id":1},
            {"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":1}
        ]"#;

        let body = batch_json.as_bytes().to_vec();
        let request_type = RequestType::from_request("POST", "/rpc".to_string(), body);

        match request_type {
            RequestType::JsonRpc(JsonRpcRequest::Batch(calls)) => {
                assert_eq!(calls.len(), 3);
                assert_eq!(calls[0].id, 1);
                assert_eq!(calls[1].id, 1);
                assert_eq!(calls[2].id, 1);
                assert_eq!(calls[0].method, "eth_blockNumber");
                assert_eq!(calls[1].method, "eth_getBalance");
                assert_eq!(calls[2].method, "eth_gasPrice");
            }
            _ => panic!("Expected batch request with duplicate IDs"),
        }
    }

    #[test]
    fn test_batch_positional_mapping() {
        let calls = vec![
            JsonRpcCall {
                jsonrpc: "2.0".to_string(),
                method: "method_a".to_string(),
                params: json!([]),
                id: 999,
            },
            JsonRpcCall {
                jsonrpc: "2.0".to_string(),
                method: "method_b".to_string(),
                params: json!([]),
                id: 999,
            },
        ];

        assert_eq!(calls[0].id, calls[1].id);
        assert_ne!(calls[0].method, calls[1].method);
    }
}
