use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcCall {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcResponse {
    pub result: Value,
}

#[derive(Debug, Clone)]
pub enum RequestType {
    Regular { path: String, method: String, body: Bytes },
    JsonRpc(JsonRpcRequest),
}

#[derive(Debug, Clone)]
pub enum JsonRpcRequest {
    Single(JsonRpcCall),
}

impl JsonRpcRequest {
    pub fn cache_key(&self, host: &str, path: &str) -> String {
        match self {
            Self::Single(call) => RequestType::create_cache_key(host, path, call),
        }
    }
}

impl RequestType {
    pub fn from_request(method: &str, path: String, body: Bytes) -> Self {
        if method == "POST" {
            if let Ok(body_str) = std::str::from_utf8(&body) {
                if let Ok(call) = serde_json::from_str::<JsonRpcCall>(body_str) {
                    return RequestType::JsonRpc(JsonRpcRequest::Single(call));
                }
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
            Self::Regular { path, .. } => vec![path.clone()],
        }
    }

    pub fn get_methods_list(&self) -> String {
        self.get_methods_for_metrics().join(",")
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::JsonRpc(_) => "application/json",
            Self::Regular { .. } => "application/json", // Default, could be enhanced based on body content
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
            params: Value::Null,
            id: 1,
        };

        let request = JsonRpcRequest::Single(call);
        let key = request.cache_key("example.com", "/rpc");

        assert_eq!(key, "example.com:POST:/rpc:eth_blockNumber");
    }
}
