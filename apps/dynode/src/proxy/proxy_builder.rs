use crate::config::{Domain, Url};

use crate::cache::RequestCache;
#[cfg(test)]
use crate::config::MetricsConfig;
use crate::metrics::Metrics;
use crate::proxy::{NodeDomain, ProxyRequestService, proxy_request::ProxyRequest};
use std::collections::HashMap;

pub struct ProxyBuilder {
    domain_configs: HashMap<String, Domain>,
    metrics: Metrics,
    cache: RequestCache,
    client: reqwest::Client,
}

impl ProxyBuilder {
    pub fn new(domain_configs: HashMap<String, Domain>, metrics: Metrics, cache: RequestCache, client: reqwest::Client) -> Self {
        Self {
            domain_configs,
            metrics,
            cache,
            client,
        }
    }

    pub fn create_for_url(&self, domain: &str, url: &Url) -> ProxyRequestService {
        let mut node_domains = HashMap::new();
        node_domains.insert(domain.to_string(), NodeDomain { url: url.clone() });

        ProxyRequestService::new(
            node_domains,
            self.domain_configs.clone(),
            self.metrics.clone(),
            self.cache.clone(),
            self.client.clone(),
        )
    }

    pub async fn handle_request_with_url(
        &self,
        request: ProxyRequest,
        url: &Url,
    ) -> Result<crate::proxy::ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let proxy_service = self.create_for_url(&request.host, url);
        let node_domain = crate::proxy::NodeDomain { url: url.clone() };
        proxy_service.handle_request(request, &node_domain).await
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
            urls: vec![create_test_url("https://primary.com")],
        }
    }

    #[test]
    fn test_simple_proxy_builder_creation() {
        let mut domain_configs = HashMap::new();
        domain_configs.insert("test.com".to_string(), create_test_domain());

        let metrics = Metrics::new(MetricsConfig::default());
        let cache = RequestCache::new(CacheConfig::default());
        let client = reqwest::Client::new();

        let builder = ProxyBuilder::new(domain_configs, metrics, cache, client);
        let url = create_test_url("https://test-node.com");
        let proxy = builder.create_for_url("test.com", &url);

        assert!(proxy.domains.contains_key("test.com"));
        assert_eq!(proxy.domains.get("test.com").unwrap().url.url, "https://test-node.com");
    }
}
