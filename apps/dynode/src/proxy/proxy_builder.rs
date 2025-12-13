use crate::cache::RequestCache;
use crate::config::HeadersConfig;
#[cfg(test)]
use crate::config::MetricsConfig;
use crate::metrics::Metrics;
use crate::proxy::{NodeDomain, ProxyRequestService, proxy_request::ProxyRequest};

#[derive(Debug, Clone)]
pub struct ProxyBuilder {
    service: ProxyRequestService,
}

impl ProxyBuilder {
    pub fn new(metrics: Metrics, cache: RequestCache, client: reqwest::Client, headers_config: HeadersConfig) -> Self {
        Self {
            service: ProxyRequestService::new(metrics, cache, client, headers_config),
        }
    }

    pub async fn handle_request(
        &self,
        request: ProxyRequest,
        node_domain: &NodeDomain,
    ) -> Result<crate::proxy::ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.service.handle_request(request, node_domain).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;
    use reqwest::header;
    use std::collections::HashMap;

    fn create_test_headers_config() -> HeadersConfig {
        HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
            domains: HashMap::new(),
        }
    }

    #[test]
    fn test_proxy_builder_creation() {
        let metrics = Metrics::new(MetricsConfig::default());
        let cache = RequestCache::new(CacheConfig::default());
        let client = reqwest::Client::new();
        let headers_config = create_test_headers_config();

        let _builder = ProxyBuilder::new(metrics, cache, client, headers_config);
    }
}
