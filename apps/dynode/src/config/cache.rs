use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use serde_serializers::duration;

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
    #[serde(default, alias = "ttl_seconds", deserialize_with = "duration::deserialize_option")]
    pub ttl: Option<Duration>,
    #[serde(default)]
    pub inflight: bool,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

impl CacheRule {
    pub fn matches_path(&self, path: &str, method: &str, body: Option<&[u8]>) -> Option<Duration> {
        self.matches_path_request(path, method, body).then_some(self.ttl).flatten()
    }

    pub fn matches_rpc_method(&self, rpc_method: &str) -> Option<Duration> {
        self.matches_rpc(rpc_method).then_some(self.ttl).flatten()
    }

    pub fn matches_path_inflight(&self, path: &str, method: &str, body: Option<&[u8]>) -> bool {
        self.inflight && self.matches_path_request(path, method, body)
    }

    pub fn matches_rpc_method_inflight(&self, rpc_method: &str) -> bool {
        self.inflight && self.matches_rpc(rpc_method)
    }

    pub(crate) fn matches_path_request(&self, path: &str, method: &str, body: Option<&[u8]>) -> bool {
        let Some(rule_method) = self.method.as_ref() else {
            return false;
        };
        if !method.eq_ignore_ascii_case(rule_method) {
            return false;
        }

        let Some(rule_path) = self.path.as_ref() else {
            return false;
        };
        let path_without_query = path.split('?').next().unwrap_or(path);
        path_without_query == rule_path && self.matches_body(body)
    }

    pub(crate) fn matches_rpc(&self, rpc_method: &str) -> bool {
        self.rpc_method.as_ref().is_some_and(|m| m == rpc_method)
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
            ttl: Some(Duration::from_secs(60)),
            inflight: false,
            params: HashMap::new(),
        };

        assert_eq!(rule.matches_path("/api/data", "GET", None), Some(Duration::from_secs(60)));
        assert_eq!(rule.matches_path("/api/data", "get", None), Some(Duration::from_secs(60))); // case insensitive
        assert_eq!(rule.matches_path("/api/data?q=1", "GET", None), Some(Duration::from_secs(60))); // strips query
        assert_eq!(rule.matches_path("/api/data", "POST", None), None);
        assert_eq!(rule.matches_path("/other", "GET", None), None);
    }

    #[test]
    fn test_matches_rpc_method() {
        let rule = CacheRule {
            path: None,
            method: None,
            rpc_method: Some("eth_blockNumber".to_string()),
            ttl: Some(Duration::from_secs(30)),
            inflight: false,
            params: HashMap::new(),
        };

        assert_eq!(rule.matches_rpc_method("eth_blockNumber"), Some(Duration::from_secs(30)));
        assert_eq!(rule.matches_rpc_method("eth_getBalance"), None);
    }

    #[test]
    fn test_matches_path_inflight() {
        let rule = CacheRule {
            path: Some("/wallet/getaccount".to_string()),
            method: Some("POST".to_string()),
            rpc_method: None,
            ttl: None,
            inflight: true,
            params: HashMap::new(),
        };

        assert!(rule.matches_path_inflight("/wallet/getaccount", "POST", Some(br#"{"address":"abc"}"#)));
        assert_eq!(rule.matches_path("/wallet/getaccount", "POST", Some(br#"{"address":"abc"}"#)), None);
    }

    #[test]
    fn test_ttl_default_none() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/wallet/getaccount",
            "method": "POST",
            "inflight": true
        }))
        .unwrap();

        assert!(rule.inflight);
        assert_eq!(rule.ttl, None);
    }

    #[test]
    fn test_ttl_duration_string() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/api/data",
            "method": "GET",
            "ttl": "1m"
        }))
        .unwrap();

        assert_eq!(rule.ttl, Some(Duration::from_secs(60)));
    }
}
