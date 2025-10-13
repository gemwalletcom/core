use crate::cache::RequestCache;
#[cfg(test)]
use crate::config::MetricsConfig;
use crate::metrics::Metrics;
use crate::proxy::{NodeDomain, ProxyRequestService, proxy_request::ProxyRequest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
use crate::config::{Domain, Url};

pub struct ProxyBuilder {
    metrics: Metrics,
    cache: RequestCache,
    client: reqwest::Client,
}

impl ProxyBuilder {
    pub fn new(metrics: Metrics, cache: RequestCache, client: reqwest::Client) -> Self {
        Self { metrics, cache, client }
    }

    pub fn create_for_domain(&self, domain: &str, node_domain: &NodeDomain) -> ProxyRequestService {
        let mut node_domains = HashMap::new();
        node_domains.insert(domain.to_string(), node_domain.clone());

        ProxyRequestService::new(
            Arc::new(RwLock::new(node_domains)),
            self.metrics.clone(),
            self.cache.clone(),
            self.client.clone(),
        )
    }

    pub async fn handle_request(
        &self,
        request: ProxyRequest,
        node_domain: &NodeDomain,
    ) -> Result<crate::proxy::ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let proxy_service = self.create_for_domain(&request.host, node_domain);
        proxy_service.handle_request(request, node_domain).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;
    use primitives::Chain;

    fn create_test_url(url: &str) -> Url {
        Url {
            url: url.to_string(),
            headers: None,
        }
    }

    fn create_test_domain() -> Domain {
        Domain {
            domain: "test.com".to_string(),
            chain: Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            overrides: None,
            urls: vec![create_test_url("https://primary.com")],
        }
    }

    #[tokio::test]
    async fn test_simple_proxy_builder_creation() {
        let metrics = Metrics::new(MetricsConfig::default());
        let cache = RequestCache::new(CacheConfig::default());
        let client = reqwest::Client::new();

        let builder = ProxyBuilder::new(metrics, cache, client);
        let url = create_test_url("https://test-node.com");
        let node_domain = NodeDomain::new(url, create_test_domain());
        let proxy = builder.create_for_domain("test.com", &node_domain);

        let domains = proxy.domains.read().await;
        assert!(domains.contains_key("test.com"));
        assert_eq!(domains.get("test.com").unwrap().url.url, "https://test-node.com");
    }
}
