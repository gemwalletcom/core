use crate::config::MetricsConfig;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
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
    config: Arc<MetricsConfig>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestLabels {
    host: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestByAgentLabels {
    host: String,
    user_agent: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestByMethodLabels {
    host: String,
    method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostStateLabels {
    host: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostCurrentStateLabels {
    host: String,
    remote_host: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct ResponseLabels {
    host: String,
    remote_host: String,
    path: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct CacheLabels {
    host: String,
    path: String,
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

        let mut registry = <Registry>::with_prefix("dynode");
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
            "Proxy requests served a response by host and path",
            proxy_response_latency.clone(),
        );
        registry.register("node_host_current", "Node current host url", node_host_current.clone());
        registry.register("node_block_latest", "Node block latest", node_block_latest.clone());
        registry.register("cache_hits", "Cache hits by host and path", cache_hits.clone());
        registry.register("cache_misses", "Cache misses by host and path", cache_misses.clone());

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
            config: Arc::new(config),
        }
    }

    fn categorize_user_agent(&self, user_agent: &str) -> String {
        for (category, patterns) in &self.config.user_agent_patterns.patterns {
            for pattern in patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(user_agent) {
                        return category.clone();
                    }
                }
            }
        }
        "unknown".to_string()
    }

    pub fn add_proxy_request(&self, host: &str, user_agent: &str) {
        self.proxy_requests.get_or_create(&ProxyRequestLabels { host: host.to_string() }).inc();

        let categorized_agent = self.categorize_user_agent(user_agent);
        self.proxy_requests_by_user_agent
            .get_or_create(&ProxyRequestByAgentLabels {
                host: host.to_string(),
                user_agent: categorized_agent,
            })
            .inc();
    }

    pub fn add_proxy_request_by_method(&self, host: &str, method: &str) {
        let method = self.truncate_path(method);
        self.proxy_requests_by_method
            .get_or_create(&ProxyRequestByMethodLabels {
                host: host.to_string(),
                method,
            })
            .inc();
    }

    fn truncate_path(&self, path: &str) -> String {
        path.split('/')
            .map(|segment| if segment.len() > 20 { ":value".to_string() } else { segment.to_string() })
            .collect::<Vec<String>>()
            .join("/")
    }

    pub fn add_proxy_response(&self, host: &str, path: &str, remote_host: &str, status: u16, latency: u128) {
        let path = self.truncate_path(path);
        self.proxy_response_latency
            .get_or_create(&ResponseLabels {
                host: host.to_string(),
                path,
                remote_host: remote_host.to_string(),
                status,
            })
            .observe(latency as f64);
    }

    pub fn set_node_host_current(&self, host: &str, remote_host: &str) {
        self.node_host_current
            .get_or_create(&HostCurrentStateLabels {
                host: host.to_string(),
                remote_host: remote_host.to_string(),
            })
            .set(1);
    }

    #[allow(dead_code)]
    pub fn set_node_block_latest(&self, host: &str, value: u64) {
        self.node_block_latest
            .get_or_create(&HostStateLabels { host: host.to_string() })
            .set(value as i64);
    }

    pub fn add_cache_hit(&self, host: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_hits.get_or_create(&CacheLabels { host: host.to_string(), path }).inc();
    }

    pub fn add_cache_miss(&self, host: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_misses.get_or_create(&CacheLabels { host: host.to_string(), path }).inc();
    }

    pub fn get_metrics(&self) -> String {
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MetricsConfig, UserAgentPatterns};

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

        metrics.add_proxy_response("example.com", "/api/v1/verylongpaththatexceedstwentycharacters", "node1.example.com", 200, 100);

        let output = metrics.get_metrics();
        assert!(output.contains("proxy_response_latency"));
    }
}
