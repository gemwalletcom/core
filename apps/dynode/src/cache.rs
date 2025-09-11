use async_trait::async_trait;
use bytes::Bytes;
use moka::future::Cache;
use moka::Expiry;
use primitives::Chain;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::config::{CacheConfig, CacheRule};

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub body: Bytes,
    pub status: u16,
    pub content_type: Option<String>,
    pub ttl_seconds: u64,
}

/// Trait for cache implementations
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, chain_type: &Chain, key: &str) -> Option<CachedResponse>;
    
    /// Set a value in the cache with TTL
    async fn set(&self, chain_type: &Chain, key: String, response: CachedResponse, ttl_seconds: u64);
    
    /// Get cache rules for a specific chain type
    fn get_cache_rules(&self, chain_type: &Chain) -> Vec<CacheRule>;
    
    /// Check if a request should be cached based on rules
    fn should_cache(&self, chain_type: &Chain, path: &str, method: &str, body: Option<&Bytes>) -> Option<u64>;
    
    /// Create a cache key from request parameters
    fn create_cache_key(host: &str, path: &str, method: &str, body: Option<&Bytes>) -> String {
        let mut key = format!("{}:{}:{}", host, method, path);
        
        if let Some(body) = body {
            Self::append_body_to_key(&mut key, body);
        }
        
        key
    }
    
    fn append_body_to_key(key: &mut String, body: &Bytes) {
        let Ok(body_str) = std::str::from_utf8(body) else {
            return;
        };
        
        if let Ok(json) = serde_json::from_str::<Value>(body_str) {
            Self::append_json_to_key(key, &json);
        } else {
            key.push(':');
            key.push_str(body_str);
        }
    }
    
    fn append_json_to_key(key: &mut String, json: &Value) {
        if let Some(rpc_method) = json.get("method").and_then(|m| m.as_str()) {
            key.push(':');
            key.push_str(rpc_method);
            
            if let Some(params) = json.get("params") {
                key.push(':');
                let params_str = serde_json::to_string(params)
                    .unwrap_or_else(|_| params.to_string());
                key.push_str(&params_str);
            }
        }
    }
}

struct CacheExpiry;

impl Expiry<String, CachedResponse> for CacheExpiry {
    fn expire_after_create(&self, _key: &String, value: &CachedResponse, _current_time: Instant) -> Option<Duration> {
        Some(Duration::from_secs(value.ttl_seconds))
    }
}

/// In-memory cache implementation using Moka
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
    
    fn check_path_rule(&self, rule: &CacheRule, path: &str, method: &str) -> Option<u64> {
        let rule_path = rule.path.as_ref()?;
        let rule_method = rule.method.as_ref()?;
        
        let path_without_query = path.split('?').next().unwrap_or(path);
        if path_without_query == rule_path && method.eq_ignore_ascii_case(rule_method) {
            Some(rule.ttl_seconds)
        } else {
            None
        }
    }
    
    fn check_rpc_rule(&self, rule: &CacheRule, body: Option<&Bytes>) -> Option<u64> {
        let rpc_method_name = rule.rpc_method.as_ref()?;
        let body = body?;
        
        let json_str = std::str::from_utf8(body).ok()?;
        let json: Value = serde_json::from_str(json_str).ok()?;
        let method = json.get("method")?.as_str()?;
        
        if method == rpc_method_name {
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
            let response_with_ttl = CachedResponse {
                ttl_seconds,
                ..response
            };
            cache.insert(key, response_with_ttl).await;
        }
    }

    fn get_cache_rules(&self, chain_type: &Chain) -> Vec<CacheRule> {
        self.config.rules.get(chain_type.as_ref()).cloned().unwrap_or_default()
    }

    fn should_cache(&self, chain_type: &Chain, path: &str, method: &str, body: Option<&Bytes>) -> Option<u64> {
        let rules = self.get_cache_rules(chain_type);
        
        for rule in rules {
            if let Some(ttl) = self.check_path_rule(&rule, path, method) {
                return Some(ttl);
            }
            
            if let Some(ttl) = self.check_rpc_rule(&rule, body) {
                return Some(ttl);
            }
        }
        
        None
    }
}

pub type RequestCache = MemoryCache;

impl RequestCache {
    pub fn create_cache_key(host: &str, path: &str, method: &str, body: Option<&Bytes>) -> String {
        let mut key = format!("{}:{}:{}", host, method, path);
        
        if let Some(body) = body {
            <MemoryCache as CacheProvider>::append_body_to_key(&mut key, body);
        }
        
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CacheConfig, CacheRule};
    use bytes::Bytes;
    use primitives::Chain;
    use std::collections::HashMap;

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
                },
                CacheRule {
                    path: None,
                    method: None,
                    rpc_method: Some("eth_blockNumber".to_string()),
                    ttl_seconds: 60,
                },
            ],
        );
        
        CacheConfig {
            max_memory_mb: 100,
            rules,
        }
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let config = create_test_config();
        let cache = RequestCache::new(config);

        let response = CachedResponse {
            body: Bytes::from("test body"),
            status: 200,
            content_type: Some("application/json".to_string()),
            ttl_seconds: 0,
        };

        cache.set(&Chain::Ethereum, "test_key".to_string(), response.clone(), 60).await;
        
        let cached = cache.get(&Chain::Ethereum, "test_key").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().status, 200);
    }

    #[test]
    fn test_cache_key_creation() {
        let key = RequestCache::create_cache_key("example.com", "/api/data", "GET", None);
        assert_eq!(key, "example.com:GET:/api/data");

        let body = Bytes::from(r#"{"method":"eth_call","params":[]}"#);
        let key = RequestCache::create_cache_key("example.com", "/", "POST", Some(&body));
        assert!(key.contains("eth_call"));
    }

    #[test]
    fn test_should_cache() {
        let config = create_test_config();
        let cache = RequestCache::new(config);

        let ttl = cache.should_cache(&Chain::Ethereum, "/api/v1/data", "GET", None);
        assert_eq!(ttl, Some(300));

        let body = Bytes::from(r#"{"method":"eth_blockNumber"}"#);
        let ttl = cache.should_cache(&Chain::Ethereum, "/", "POST", Some(&body));
        assert_eq!(ttl, Some(60));

        let ttl = cache.should_cache(&Chain::Ethereum, "/unknown", "GET", None);
        assert_eq!(ttl, None);
    }
}