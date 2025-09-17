use std::collections::HashMap;

use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub max_memory_mb: usize,
    #[serde(default)]
    pub rules: HashMap<String, Vec<CacheRule>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheRule {
    pub path: Option<String>,
    pub method: Option<String>,
    pub rpc_method: Option<String>,
    pub ttl_seconds: u64,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

impl CacheRule {
    pub fn matches_body(&self, body: Option<&Bytes>) -> bool {
        if self.params.is_empty() {
            return true;
        }

        let Some(body_bytes) = body else {
            return false;
        };

        let Ok(value) = serde_json::from_slice::<Value>(body_bytes) else {
            return false;
        };

        let Some(object) = value.as_object() else {
            return false;
        };

        self.params.iter().all(|(key, expected)| {
            object
                .get(key)
                .and_then(|value| value.as_str())
                .map(|actual| actual == expected)
                .unwrap_or(false)
        })
    }
}
