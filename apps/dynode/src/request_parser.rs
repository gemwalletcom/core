use bytes::Bytes;
use hyper::Method;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParsedRequest {
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub method: Method,
    #[allow(dead_code)]
    pub host: String,
    pub rpc_method: Option<String>,
    #[allow(dead_code)]
    pub rpc_params: Option<Value>,
}

impl ParsedRequest {
    pub fn new(host: &str, path: &str, method: &Method, body: Option<&Bytes>) -> Self {
        let mut parsed = Self {
            path: path.to_string(),
            method: method.clone(),
            host: host.to_string(),
            rpc_method: None,
            rpc_params: None,
        };
        
        if let Some(body) = body {
            if let Ok(json_str) = std::str::from_utf8(body) {
                if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                    parsed.rpc_method = json.get("method")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string());
                    
                    parsed.rpc_params = json.get("params").cloned();
                }
            }
        }
        
        parsed
    }
    
    #[allow(dead_code)]
    pub fn is_rpc(&self) -> bool {
        self.rpc_method.is_some()
    }
    
    #[allow(dead_code)]
    pub fn cache_key_suffix(&self) -> String {
        if let Some(ref rpc_method) = self.rpc_method {
            if let Some(ref params) = self.rpc_params {
                format!("{}:{}", rpc_method, serde_json::to_string(params).unwrap_or_default())
            } else {
                rpc_method.clone()
            }
        } else {
            String::new()
        }
    }
}