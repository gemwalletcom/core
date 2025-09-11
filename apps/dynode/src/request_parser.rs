use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    method: String,
    #[allow(dead_code)]
    params: Value,
}

pub fn extract_rpc_method(body: &Bytes) -> Option<String> {
    let body_str = std::str::from_utf8(body).ok()?;
    serde_json::from_str::<JsonRpcRequest>(body_str).ok().map(|req| req.method)
}
