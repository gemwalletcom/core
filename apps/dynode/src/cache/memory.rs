use crate::config::{CacheConfig, CacheRule};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, RequestType};
use crate::proxy::CachedResponse;
use primitives::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::CacheProvider;
use super::types::CacheEntry;

#[derive(Debug, Clone)]
pub struct MemoryCache {
    caches: Arc<HashMap<String, Arc<RwLock<HashMap<String, CacheEntry>>>>>,
    config: CacheConfig,
}

impl MemoryCache {
    pub fn new(config: CacheConfig) -> Self {
        let mut caches = HashMap::new();
        for chain_name in config.rules.keys() {
            caches.insert(chain_name.clone(), Arc::new(RwLock::new(HashMap::new())));
        }
        Self { caches: Arc::new(caches), config }
    }

    fn max_size_per_chain(&self) -> usize {
        let chain_count = self.caches.len().max(1);
        (self.config.max_memory_mb * 1_000_000) / chain_count
    }

    fn evict_if_needed(cache: &mut HashMap<String, CacheEntry>, max_size: usize) {
        let mut size = 0;
        cache.retain(|_, entry| {
            if entry.is_expired() {
                false
            } else {
                size += entry.size();
                true
            }
        });

        if size <= max_size {
            return;
        }

        let mut valid_entries: Vec<_> = cache.iter().map(|(key, entry)| (key.clone(), entry.created_at)).collect();
        valid_entries.sort_unstable_by_key(|(_, created)| *created);

        for (key, _) in valid_entries {
            if size <= max_size {
                break;
            }
            if let Some(entry) = cache.remove(&key) {
                size -= entry.size();
            }
        }
    }

    fn get_cache_rules(&self, chain: &Chain) -> &[CacheRule] {
        static EMPTY: &[CacheRule] = &[];
        self.config.rules.get(chain.as_ref()).map(|v| v.as_slice()).unwrap_or(EMPTY)
    }
}

impl CacheProvider for MemoryCache {
    async fn get(&self, chain: &Chain, key: &str) -> Option<CachedResponse> {
        let cache = self.caches.get(chain.as_ref())?;
        let read_guard = cache.read().await;
        let entry = read_guard.get(key)?;
        if entry.is_expired() {
            drop(read_guard);
            cache.write().await.remove(key);
            return None;
        }
        Some(entry.response.clone())
    }

    async fn set(&self, chain: &Chain, key: String, response: CachedResponse, ttl_seconds: u64) {
        if let Some(cache) = self.caches.get(chain.as_ref()) {
            let entry = CacheEntry::new(response, ttl_seconds);
            let mut guard = cache.write().await;
            guard.insert(key, entry);
            Self::evict_if_needed(&mut guard, self.max_size_per_chain());
        }
    }

    fn should_cache(&self, chain: &Chain, path: &str, method: &str, body: Option<&[u8]>) -> Option<u64> {
        self.get_cache_rules(chain).iter().find_map(|rule| rule.matches_path(path, method, body))
    }

    fn should_cache_request(&self, chain: &Chain, request_type: &RequestType) -> Option<u64> {
        for rule in self.get_cache_rules(chain) {
            match request_type {
                RequestType::Regular { path, method, body } => {
                    if let Some(ttl) = rule.matches_path(path, method, Some(body.as_slice())) {
                        return Some(ttl);
                    }
                }
                RequestType::JsonRpc(JsonRpcRequest::Single(call)) => {
                    if let Some(ttl) = rule.matches_rpc_method(&call.method) {
                        return Some(ttl);
                    }
                }
                RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => {
                    return None;
                }
            }
        }

        None
    }

    fn should_cache_call(&self, chain: &Chain, call: &JsonRpcCall) -> Option<u64> {
        self.get_cache_rules(chain).iter().find_map(|rule| rule.matches_rpc_method(&call.method))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::StatusCode;
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

        let response = CachedResponse::new(b"test".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), 60);
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
    fn test_should_cache_with_params() {
        let mut config = create_test_config();
        if let Some(rules) = config.rules.get_mut("ethereum") {
            let mut params = HashMap::new();
            params.insert("type".to_string(), serde_json::json!("metaAndAssetCtxs"));

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

        let matching_body = r#"{"type":"metaAndAssetCtxs"}"#.as_bytes().to_vec();
        let ttl = cache.should_cache(&chain, "/info", "POST", Some(matching_body.as_slice()));
        assert_eq!(ttl, Some(200));

        let non_matching_body = r#"{"type":"other"}"#.as_bytes().to_vec();
        let ttl = cache.should_cache(&chain, "/info", "POST", Some(non_matching_body.as_slice()));
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

    #[test]
    fn test_should_cache_with_function_params() {
        let mut config = create_test_config();
        let mut aptos_rules = Vec::new();
        let mut params = HashMap::new();
        params.insert("function".to_string(), serde_json::json!("0x1::delegation_pool::operator_commission_percentage"));

        aptos_rules.push(CacheRule {
            path: Some("/v1/view".to_string()),
            method: Some("POST".to_string()),
            rpc_method: None,
            ttl_seconds: 3600,
            params,
        });

        config.rules.insert("aptos".to_string(), aptos_rules);
        let cache = MemoryCache::new(config);
        let chain = Chain::Aptos;

        let body1 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache(&chain, "/v1/view", "POST", Some(body1.as_slice()));
        assert_eq!(ttl, Some(3600));

        let body2 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache(&chain, "/v1/view", "POST", Some(body2.as_slice()));
        assert_eq!(ttl, Some(3600));

        let body3 = r#"{
            "function": "0x1::other_module::other_function",
            "type_arguments": [],
            "arguments": ["0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache(&chain, "/v1/view", "POST", Some(body3.as_slice()));
        assert_eq!(ttl, None);
    }

    #[tokio::test]
    async fn test_eviction() {
        let mut rules = HashMap::new();
        rules.insert("ethereum".to_string(), vec![]);

        let config = CacheConfig {
            max_memory_mb: 0, // Force eviction on any insert
            rules,
        };
        let cache = MemoryCache::new(config);
        let chain = Chain::Ethereum;

        // Insert first entry
        let response1 = CachedResponse::new(b"first".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), 60);
        cache.set(&chain, "key1".to_string(), response1, 60).await;

        // Insert second entry - should evict first due to max_memory_mb = 0
        let response2 = CachedResponse::new(b"second".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), 60);
        cache.set(&chain, "key2".to_string(), response2, 60).await;

        // First key should be evicted
        assert!(cache.get(&chain, "key1").await.is_none());
        // Second key might also be evicted depending on size, but let's just verify eviction happened
    }
}
