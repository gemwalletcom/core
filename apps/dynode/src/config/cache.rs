use std::collections::HashMap;

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
    pub params: HashMap<String, Value>,
}

impl CacheRule {
    pub fn matches_path(&self, path: &str, method: &str, body: Option<&[u8]>) -> Option<u64> {
        let rule_method = self.method.as_ref()?;
        if !method.eq_ignore_ascii_case(rule_method) {
            return None;
        }

        let rule_path = self.path.as_ref()?;
        let path_without_query = path.split('?').next().unwrap_or(path);

        if path_without_query == rule_path && self.matches_body(body) {
            Some(self.ttl_seconds)
        } else {
            None
        }
    }

    pub fn matches_rpc_method(&self, rpc_method: &str) -> Option<u64> {
        if self.rpc_method.as_ref().is_some_and(|m| m == rpc_method) {
            Some(self.ttl_seconds)
        } else {
            None
        }
    }

    fn matches_body(&self, body: Option<&[u8]>) -> bool {
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

        self.params.iter().all(|(key, expected)| object.get(key).map(|actual| actual == expected).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_path() {
        let rule = CacheRule {
            path: Some("/api/data".to_string()),
            method: Some("GET".to_string()),
            rpc_method: None,
            ttl_seconds: 60,
            params: HashMap::new(),
        };

        assert_eq!(rule.matches_path("/api/data", "GET", None), Some(60));
        assert_eq!(rule.matches_path("/api/data", "get", None), Some(60)); // case insensitive
        assert_eq!(rule.matches_path("/api/data?q=1", "GET", None), Some(60)); // strips query
        assert_eq!(rule.matches_path("/api/data", "POST", None), None);
        assert_eq!(rule.matches_path("/other", "GET", None), None);
    }

    #[test]
    fn test_matches_rpc_method() {
        let rule = CacheRule {
            path: None,
            method: None,
            rpc_method: Some("eth_blockNumber".to_string()),
            ttl_seconds: 30,
            params: HashMap::new(),
        };

        assert_eq!(rule.matches_rpc_method("eth_blockNumber"), Some(30));
        assert_eq!(rule.matches_rpc_method("eth_getBalance"), None);
    }
}
