#[cfg(test)]
mod cache_tests {
    use super::super::cache::*;
    use super::super::config::{CacheConfig, CacheRule};
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

#[cfg(test)]
mod chain_service_tests {
    use super::super::chain_service::ChainService;
    use primitives::Chain;

    #[tokio::test]
    async fn test_unsupported_chain_returns_error() {
        let service = ChainService::new(Chain::Stellar, "https://example.com".to_string());
        let result = service.get_block_number().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "not implemented");
    }
}

#[cfg(test)]
mod metrics_tests {
    use super::super::metrics::*;
    use super::super::config::{MetricsConfig, UserAgentPatterns};

    #[test]
    fn test_cache_metrics_labels() {
        let config = MetricsConfig {
            user_agent_patterns: UserAgentPatterns::default(),
        };
        let metrics = Metrics::new(config);

        metrics.add_cache_hit("example.com", "eth_blockNumber");
        metrics.add_cache_miss("example.com", "/api/v1/data");

        let output = metrics.get_metrics();
        assert!(output.contains("cache_hits"));
        assert!(output.contains("cache_misses"));
    }

    #[test]
    fn test_path_truncation() {
        let config = MetricsConfig {
            user_agent_patterns: UserAgentPatterns::default(),
        };
        let metrics = Metrics::new(config);

        metrics.add_proxy_response(
            "example.com",
            "/api/v1/verylongpaththatexceedstwentycharacters",
            "node1.example.com",
            200,
            100,
        );

        let output = metrics.get_metrics();
        assert!(output.contains("proxy_response_latency"));
    }
}