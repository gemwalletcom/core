use crate::config::MetricsConfig;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;
use regex::Regex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
    proxy_requests: Family<ProxyRequestLabels, Counter>,
    proxy_requests_by_user_agent: Family<ProxyRequestByAgentLabels, Gauge>,
    proxy_requests_by_method: Family<ProxyRequestByMethodLabels, Counter>,
    proxy_response_latency: Family<ResponseLabels, Histogram>,
    node_host_current: Family<HostCurrentStateLabels, Gauge>,
    #[allow(dead_code)]
    node_block_latest: Family<HostStateLabels, Gauge>,
    cache_hits: Family<CacheLabels, Counter>,
    cache_misses: Family<CacheLabels, Counter>,
    node_switches: Family<NodeSwitchLabels, Counter>,
    config: Arc<MetricsConfig>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestLabels {
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestByAgentLabels {
    chain: String,
    user_agent: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestByMethodLabels {
    chain: String,
    method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostStateLabels {
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostCurrentStateLabels {
    chain: String,
    host: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct ResponseLabels {
    chain: String,
    host: String,
    path: String,
    method: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct CacheLabels {
    chain: String,
    path: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct NodeSwitchLabels {
    chain: String,
    old_host: String,
    new_host: String,
}

impl Metrics {
    pub fn new(config: MetricsConfig) -> Self {
        let proxy_requests = Family::<ProxyRequestLabels, Counter>::default();
        let proxy_requests_by_user_agent = Family::<ProxyRequestByAgentLabels, Gauge>::default();
        let proxy_requests_by_method = Family::<ProxyRequestByMethodLabels, Counter>::default();
        let proxy_response_latency = Family::<ResponseLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(50.0, 1.44, 12)));
        let node_host_current = Family::<HostCurrentStateLabels, Gauge>::default();
        let node_block_latest = Family::<HostStateLabels, Gauge>::default();
        let cache_hits = Family::<CacheLabels, Counter>::default();
        let cache_misses = Family::<CacheLabels, Counter>::default();
        let node_switches = Family::<NodeSwitchLabels, Counter>::default();

        let mut registry = Registry::with_prefix(&config.prefix);
        registry.register("proxy_requests", "Proxy requests by host", proxy_requests.clone());
        registry.register(
            "proxy_requests_by_user_agent_total",
            "Proxy requests by host and user agent",
            proxy_requests_by_user_agent.clone(),
        );
        registry.register(
            "proxy_requests_by_method",
            "Proxy requests by host and method (HTTP path or RPC method)",
            proxy_requests_by_method.clone(),
        );
        registry.register(
            "proxy_response_latency",
            "Proxy responses by host, path, method, and status",
            proxy_response_latency.clone(),
        );
        registry.register("node_host_current", "Node current host url", node_host_current.clone());
        registry.register("node_block_latest", "Node block latest", node_block_latest.clone());
        registry.register("cache_hits", "Cache hits by host and path", cache_hits.clone());
        registry.register("cache_misses", "Cache misses by host and path", cache_misses.clone());
        registry.register("node_switches_total", "Node switches by chain and host", node_switches.clone());

        Self {
            registry: Arc::new(registry),
            proxy_requests,
            proxy_requests_by_user_agent,
            proxy_requests_by_method,
            proxy_response_latency,
            node_host_current,
            node_block_latest,
            cache_hits,
            cache_misses,
            node_switches,
            config: Arc::new(config),
        }
    }

    pub fn add_proxy_request(&self, chain: &str, user_agent: &str) {
        self.proxy_requests.get_or_create(&ProxyRequestLabels { chain: chain.to_string() }).inc();

        let user_agent = self.categorize_user_agent(user_agent);
        self.proxy_requests_by_user_agent
            .get_or_create(&ProxyRequestByAgentLabels {
                chain: chain.to_string(),
                user_agent,
            })
            .inc();
    }

    pub fn add_proxy_request_by_method(&self, chain: &str, method: &str) {
        let method = self.truncate_method(method);
        self.proxy_requests_by_method
            .get_or_create(&ProxyRequestByMethodLabels {
                chain: chain.to_string(),
                method,
            })
            .inc();
    }

    pub fn add_proxy_request_batch(&self, chain: &str, user_agent: &str, methods: &[String]) {
        self.proxy_requests.get_or_create(&ProxyRequestLabels { chain: chain.to_string() }).inc();

        let user_agent = self.categorize_user_agent(user_agent);
        self.proxy_requests_by_user_agent
            .get_or_create(&ProxyRequestByAgentLabels {
                chain: chain.to_string(),
                user_agent,
            })
            .inc();

        for method in methods {
            let method = self.truncate_method(method);
            self.proxy_requests_by_method
                .get_or_create(&ProxyRequestByMethodLabels {
                    chain: chain.to_string(),
                    method,
                })
                .inc();
        }
    }

    pub fn add_proxy_response(&self, chain: &str, path: &str, method: &str, host: &str, status: u16, latency: u128) {
        let path = self.truncate_path(path);
        let method = self.truncate_method(method);
        self.proxy_response_latency
            .get_or_create(&ResponseLabels {
                chain: chain.to_string(),
                path,
                method,
                host: host.to_string(),
                status,
            })
            .observe(latency as f64);
    }

    pub fn set_node_host_current(&self, chain: &str, host: &str) {
        self.node_host_current
            .get_or_create(&HostCurrentStateLabels {
                chain: chain.to_string(),
                host: host.to_string(),
            })
            .set(1);
    }

    #[allow(dead_code)]
    pub fn set_node_block_latest(&self, chain: &str, value: u64) {
        self.node_block_latest
            .get_or_create(&HostStateLabels { chain: chain.to_string() })
            .set(value as i64);
    }

    pub fn add_cache_hit(&self, chain: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_hits.get_or_create(&CacheLabels { chain: chain.to_string(), path }).inc();
    }

    pub fn add_cache_miss(&self, chain: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_misses.get_or_create(&CacheLabels { chain: chain.to_string(), path }).inc();
    }

    pub fn add_node_switch(&self, chain: &str, old_host: &str, new_host: &str) {
        self.node_switches
            .get_or_create(&NodeSwitchLabels {
                chain: chain.to_string(),
                old_host: old_host.to_string(),
                new_host: new_host.to_string(),
            })
            .inc();
    }

    pub fn get_metrics(&self) -> String {
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).expect("failed to encode metrics");
        buffer
    }

    fn truncate_method(&self, method: &str) -> String {
        self.truncate_path(method)
    }

    fn truncate_path(&self, path: &str) -> String {
        let (path_part, query_part) = path.split_once('?').map(|(p, q)| (p, Some(q))).unwrap_or((path, None));

        let truncated_path = path_part
            .split('/')
            .map(|segment| {
                if segment.is_empty() {
                    segment.to_string()
                } else if segment.chars().all(|c| c.is_ascii_digit()) {
                    ":number".to_string()
                } else if segment.len() > 20 {
                    ":value".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("/");

        if let Some(query) = query_part {
            let truncated_query = query
                .split('&')
                .map(|param| {
                    if let Some((key, _)) = param.split_once('=') {
                        format!("{}=:v", key)
                    } else {
                        param.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("&");
            format!("{}?{}", truncated_path, truncated_query)
        } else {
            truncated_path
        }
    }

    fn categorize_user_agent(&self, user_agent: &str) -> String {
        for (category, patterns) in &self.config.user_agent_patterns.patterns {
            for pattern in patterns {
                if let Ok(re) = Regex::new(pattern)
                    && re.is_match(user_agent)
                {
                    return category.clone();
                }
            }
        }
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MetricsConfig, UserAgentPatterns};
    use std::collections::HashMap;

    fn create_test_metrics() -> Metrics {
        let config = MetricsConfig {
            prefix: "test".to_string(),
            user_agent_patterns: UserAgentPatterns { patterns: HashMap::new() },
        };
        Metrics::new(config)
    }

    #[test]
    fn test_truncate_path() {
        let metrics = create_test_metrics();

        let test_cases = vec![
            ("/api/v1/verylongsegmentthatisgreaterthan20characters/data", "/api/v1/:value/data"),
            ("/block/12345/transactions", "/block/:number/transactions"),
            ("/block/12345/tx/67890", "/block/:number/tx/:number"),
            ("/api/123456/verylongsegmentthatisgreaterthan20chars/data", "/api/:number/:value/data"),
            ("/api/v1/data", "/api/v1/data"),
            ("/api//data", "/api//data"),
            ("/api/v2/block/5897744?page=1", "/api/v2/block/:number?page=:v"),
            ("/api/v2/block/5897744?page=1&limit=10", "/api/v2/block/:number?page=:v&limit=:v"),
        ];

        for (input, expected) in test_cases {
            let result = metrics.truncate_path(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
