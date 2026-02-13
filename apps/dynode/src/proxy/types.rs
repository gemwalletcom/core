use crate::jsonrpc_types::JsonRpcCall;
use std::str::from_utf8;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub body: Vec<u8>,
    pub status: u16,
    pub content_type: String,
    pub ttl: Duration,
}

impl CachedResponse {
    pub fn new(body: Vec<u8>, status: u16, content_type: String, ttl: Duration) -> Self {
        Self { body, status, content_type, ttl }
    }

    pub fn to_jsonrpc_response(&self, original_call: &JsonRpcCall) -> Vec<u8> {
        let result_str = from_utf8(&self.body).unwrap_or("null");
        format!(r#"{{"jsonrpc":"{}","result":{},"id":{}}}"#, original_call.jsonrpc, result_str, original_call.id).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::StatusCode;

    #[test]
    fn test_to_jsonrpc_response() {
        let response = CachedResponse::new(
            br#"{"value":123}"#.to_vec(),
            StatusCode::OK.as_u16(),
            JSON_CONTENT_TYPE.to_string(),
            Duration::from_secs(60),
        );
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: 42,
        };

        let result = response.to_jsonrpc_response(&call);
        let result_str = from_utf8(&result).unwrap();

        assert_eq!(result_str, r#"{"jsonrpc":"2.0","result":{"value":123},"id":42}"#);
    }

    #[test]
    fn test_to_jsonrpc_response_invalid_utf8() {
        let response = CachedResponse::new(vec![0xff, 0xfe], StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), Duration::from_secs(60));
        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: serde_json::json!([]),
            id: 1,
        };

        let result = response.to_jsonrpc_response(&call);
        let result_str = from_utf8(&result).unwrap();

        assert_eq!(result_str, r#"{"jsonrpc":"2.0","result":null,"id":1}"#);
    }
}
