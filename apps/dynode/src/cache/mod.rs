use crate::config::{CacheConfig, CacheRule};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, RequestType};
use async_trait::async_trait;
use bytes::Bytes;
use moka::future::Cache;
use moka::Expiry;
use primitives::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub body: Bytes,
    pub status: u16,
    pub content_type: String,
    pub ttl_seconds: u64,
}

impl CachedResponse {
    pub fn new(body: Bytes, status: u16, content_type: String, ttl_seconds: u64) -> Self {
        Self {
            body,
            status,
            content_type,
            ttl_seconds,
        }
    }

    pub fn to_jsonrpc_response(&self, original_call: &JsonRpcCall) -> Bytes {
        let result_str = std::str::from_utf8(&self.body).unwrap_or("null");
        Bytes::from(format!(
            r#"{{"jsonrpc":"{}","result":{},"id":{}}}"#,
            original_call.jsonrpc, result_str, original_call.id
        ))
    }
}

#[async_trait]
pub trait CacheProvider: Send + Sync {
    async fn get(&self, chain_type: &Chain, key: &str) -> Option<CachedResponse>;
    async fn set(&self, chain_type: &Chain, key: String, response: CachedResponse, ttl_seconds: u64);
    fn get_cache_rules(&self, chain_type: &Chain) -> Vec<CacheRule>;
    fn should_cache(&self, chain_type: &Chain, path: &str, method: &str, body: Option<&Bytes>) -> Option<u64>;
    fn should_cache_request(&self, chain_type: &Chain, request_type: &RequestType) -> Option<u64>;
    fn should_cache_call(&self, chain_type: &Chain, call: &JsonRpcCall) -> Option<u64>;
}

struct CacheExpiry;

impl Expiry<String, CachedResponse> for CacheExpiry {
    fn expire_after_create(&self, _key: &String, value: &CachedResponse, _current_time: Instant) -> Option<Duration> {
        if value.ttl_seconds == 0 {
            None
        } else {
            Some(Duration::from_secs(value.ttl_seconds))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryCache {
    caches: Arc<HashMap<String, Cache<String, CachedResponse>>>,
    config: CacheConfig,
}

impl MemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        let mut caches = HashMap::new();

        for chain_name in config.rules.keys() {
            let cache = Self::create_chain_cache(&config);
            caches.insert(chain_name.clone(), cache);
        }

        Self {
            caches: Arc::new(caches),
            config,
        }
    }

    fn create_chain_cache(config: &CacheConfig) -> Cache<String, CachedResponse> {
        let chain_count = config.rules.len().max(1);
        let max_memory_per_chain = (config.max_memory_mb * 1_000_000) / chain_count;

        Cache::builder()
            .max_capacity(max_memory_per_chain as u64)
            .weigher(|_key: &String, value: &CachedResponse| -> u32 {
                let body_size = value.body.len();
                let overhead = 1024;
                (body_size + overhead).min(u32::MAX as usize) as u32
            })
            .expire_after(CacheExpiry)
            .build()
    }

    fn check_path_rule(&self, rule: &CacheRule, path: &str, method: &str, body: Option<&Bytes>) -> Option<u64> {
        let rule_method = rule.method.as_ref()?;
        if !method.eq_ignore_ascii_case(rule_method) {
            return None;
        }

        let rule_path = rule.path.as_ref()?;
        let path_without_query = path.split('?').next().unwrap_or(path);

        if path_without_query == rule_path && rule.matches_body(body) {
            Some(rule.ttl_seconds)
        } else {
            None
        }
    }
}

#[async_trait]
impl CacheProvider for MemoryCache {
    async fn get(&self, chain_type: &Chain, key: &str) -> Option<CachedResponse> {
        self.caches.get(chain_type.as_ref())?.get(key).await
    }

    async fn set(&self, chain_type: &Chain, key: String, response: CachedResponse, ttl_seconds: u64) {
        if let Some(cache) = self.caches.get(chain_type.as_ref()) {
            let response_with_ttl = CachedResponse { ttl_seconds, ..response };
            cache.insert(key, response_with_ttl).await;
        }
    }

    fn get_cache_rules(&self, chain_type: &Chain) -> Vec<CacheRule> {
        self.config.rules.get(chain_type.as_ref()).cloned().unwrap_or_default()
    }

    fn should_cache(&self, chain_type: &Chain, path: &str, method: &str, body: Option<&Bytes>) -> Option<u64> {
        let rules = self.get_cache_rules(chain_type);

        for rule in rules {
            if let Some(ttl) = self.check_path_rule(&rule, path, method, body) {
                return Some(ttl);
            }
        }

        None
    }

    fn should_cache_request(&self, chain_type: &Chain, request_type: &RequestType) -> Option<u64> {
        let rules = self.get_cache_rules(chain_type);

        for rule in rules {
            match request_type {
                RequestType::Regular { path, method, body } => {
                    if let Some(ttl) = self.check_path_rule(&rule, path, method, Some(body)) {
                        return Some(ttl);
                    }
                }
                RequestType::JsonRpc(JsonRpcRequest::Single(call)) => {
                    if let Some(rpc_method_name) = &rule.rpc_method {
                        if call.method == *rpc_method_name {
                            return Some(rule.ttl_seconds);
                        }
                    }
                }
                RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => {
                    if rule.rpc_method.is_some() {
                        return Some(rule.ttl_seconds);
                    }
                }
            }
        }

        None
    }

    fn should_cache_call(&self, chain_type: &Chain, call: &JsonRpcCall) -> Option<u64> {
        let rules = self.get_cache_rules(chain_type);

        for rule in rules {
            if let Some(rpc_method_name) = &rule.rpc_method {
                if call.method == *rpc_method_name {
                    return Some(rule.ttl_seconds);
                }
            }
        }

        None
    }
}

pub type RequestCache = MemoryCache;

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use primitives::Chain;
    use std::collections::HashMap;
    use std::time::Instant;

    fn create_test_config() -> CacheConfig {
        let mut rules = HashMap::new();
        rules.insert(
            "ethereum".to_string(),
            vec![
                CacheRule {
                    path: Some("/api/v1/data".to_string()),
                    method: Some("GET".to_string()),
                    rpc_method: None,
                    ttl_seconds: 300,
                    params: HashMap::new(),
                },
                CacheRule {
                    path: None,
                    method: None,
                    rpc_method: Some("eth_blockNumber".to_string()),
                    ttl_seconds: 60,
                    params: HashMap::new(),
                },
            ],
        );

        CacheConfig { max_memory_mb: 64, rules }
    }

    #[tokio::test]
    async fn test_set_and_get_cache() {
        let config = create_test_config();
        let cache = MemoryCache::new(config);
        let chain = Chain::Ethereum;

        let response = CachedResponse::new(Bytes::from("test"), 200, "application/json".to_string(), 60);
        cache.set(&chain, "test_key".to_string(), response.clone(), 60).await;

        let cached = cache.get(&chain, "test_key").await.unwrap();
        assert_eq!(cached.body, response.body);
        assert_eq!(cached.status, response.status);
    }

    #[test]
    fn test_should_cache_path_rule() {
        let config = create_test_config();
        let cache = MemoryCache::new(config.clone());
        let chain = Chain::Ethereum;

        let ttl = cache.should_cache(&chain, "/api/v1/data", "GET", None);
        assert_eq!(ttl, Some(300));

        let ttl = cache.should_cache(&chain, "/api/v1/data", "POST", None);
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_cache_expiry_without_ttl() {
        let expiry = CacheExpiry;
        let response = CachedResponse::new(Bytes::from("no-expire"), 200, "application/json".to_string(), 0);
        let key = "no_expire_key".to_string();

        let expiry_duration = expiry.expire_after_create(&key, &response, Instant::now());
        assert!(expiry_duration.is_none());
    }

    #[test]
    fn test_should_cache_with_params() {
        let mut config = create_test_config();
        if let Some(rules) = config.rules.get_mut("ethereum") {
            let mut params = HashMap::new();
            params.insert("type".to_string(), "metaAndAssetCtxs".to_string());

            rules.push(CacheRule {
                path: Some("/info".to_string()),
                method: Some("POST".to_string()),
                rpc_method: None,
                ttl_seconds: 200,
                params,
            });
        }

        let cache = MemoryCache::new(config);
        let chain = Chain::Ethereum;

        let matching_body = Bytes::from(r#"{"type":"metaAndAssetCtxs"}"#);
        let ttl = cache.should_cache(&chain, "/info", "POST", Some(&matching_body));
        assert_eq!(ttl, Some(200));

        let non_matching_body = Bytes::from(r#"{"type":"other"}"#);
        let ttl = cache.should_cache(&chain, "/info", "POST", Some(&non_matching_body));
        assert_eq!(ttl, None);

        let ttl = cache.should_cache(&chain, "/info", "POST", None);
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_should_cache_request() {
        let config = create_test_config();
        let cache = MemoryCache::new(config.clone());
        let chain = Chain::Ethereum;

        let request = RequestType::JsonRpc(JsonRpcRequest::Single(JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: 1,
        }));

        let ttl = cache.should_cache_request(&chain, &request);
        assert_eq!(ttl, Some(60));
    }

    #[test]
    fn test_should_cache_call() {
        let config = create_test_config();
        let cache = MemoryCache::new(config.clone());
        let chain = Chain::Ethereum;

        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: 1,
        };

        let ttl = cache.should_cache_call(&chain, &call);
        assert_eq!(ttl, Some(60));
    }
}
