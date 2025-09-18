use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    method: String,
    #[allow(dead_code)]
    params: Value,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RpcRequestType {
    Single(JsonRpcRequest),
    Batch(Vec<JsonRpcRequest>),
}

pub fn extract_rpc_methods(body: &[u8]) -> Vec<String> {
    let body_str = match std::str::from_utf8(body) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    match serde_json::from_str::<RpcRequestType>(body_str) {
        Ok(RpcRequestType::Single(req)) => vec![req.method],
        Ok(RpcRequestType::Batch(reqs)) => reqs.into_iter().map(|req| req.method).collect(),
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_single_rpc_method() {
        let single_request = r#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#;
        let methods = extract_rpc_methods(single_request.as_bytes());
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0], "eth_blockNumber");
    }

    #[test]
    fn test_extract_batch_rpc_methods() {
        let batch_request = r#"[
            {"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1},
            {"jsonrpc":"2.0","method":"eth_getBalance","params":["0x123",false],"id":2},
            {"jsonrpc":"2.0","method":"eth_call","params":[{},"latest"],"id":3}
        ]"#;
        let methods = extract_rpc_methods(batch_request.as_bytes());
        assert_eq!(methods.len(), 3);
        assert_eq!(methods[0], "eth_blockNumber");
        assert_eq!(methods[1], "eth_getBalance");
        assert_eq!(methods[2], "eth_call");
    }

    #[test]
    fn test_extract_invalid_json() {
        let invalid_json = r#"not valid json"#;
        let methods = extract_rpc_methods(invalid_json.as_bytes());
        assert_eq!(methods.len(), 0);
    }

    #[test]
    fn test_extract_empty_batch() {
        let empty_batch = r#"[]"#;
        let methods = extract_rpc_methods(empty_batch.as_bytes());
        assert_eq!(methods.len(), 0);
    }
}
