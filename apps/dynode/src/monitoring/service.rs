use std::error::Error;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};

use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::cache::RequestCache;
use crate::config::{CacheConfig, Domain, NodeMonitoringConfig, RequestConfig, RetryConfig};
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::monitoring::domain_resolution::DomainResolution;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_builder::ProxyBuilder;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::response_builder::ResponseBuilder;
use crate::proxy::{NodeDomain, ProxyResponse};
use primitives::{ResponseError, response::ErrorDetail};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
    pub nodes: Arc<RwLock<HashMap<String, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub cache: RequestCache,
    pub monitoring_config: NodeMonitoringConfig,
    pub retry_config: RetryConfig,
    pub request_config: RequestConfig,
    pub http_client: reqwest::Client,
}

impl NodeService {
    pub fn new(
        domains: HashMap<String, Domain>,
        metrics: Metrics,
        cache_config: CacheConfig,
        monitoring_config: NodeMonitoringConfig,
        retry_config: RetryConfig,
        request_config: RequestConfig,
    ) -> Self {
        let mut hash_map: HashMap<String, NodeDomain> = HashMap::new();

        for (key, domain) in domains.clone() {
            let url = domain.urls.first().unwrap().clone();
            hash_map.insert(key, NodeDomain::new(url, domain.clone()));
        }

        let http_client = gem_client::builder().timeout(Duration::from_millis(request_config.timeout)).build().unwrap();

        Self {
            domains,
            nodes: Arc::new(RwLock::new(hash_map)),
            metrics: Arc::new(metrics),
            cache: RequestCache::new(cache_config),
            monitoring_config,
            retry_config,
            request_config,
            http_client,
        }
    }

    pub async fn get_node_domain(nodes: &Arc<RwLock<HashMap<String, NodeDomain>>>, domain: String) -> Option<NodeDomain> {
        (nodes.read().await).get(&domain).cloned()
    }

    pub async fn update_node_domain(nodes: &Arc<RwLock<HashMap<String, NodeDomain>>>, domain: String, node_domain: NodeDomain) {
        let mut map = nodes.write().await;
        map.insert(domain, node_domain);
    }

    pub fn resolve_chain(&self, host: &str, path: &str) -> Option<DomainResolution> {
        if let Some(domain) = self.domains.get(host) {
            return Some(DomainResolution::Host(domain.chain));
        }

        let chain_from_path = path.trim_start_matches('/').split('/').next()?;
        primitives::Chain::from_str(chain_from_path).ok().map(DomainResolution::Path)
    }

    pub async fn start_monitoring(&self) {
        let monitor = NodeMonitor::new(
            self.domains.clone(),
            Arc::clone(&self.nodes),
            Arc::clone(&self.metrics),
            self.monitoring_config.clone(),
        );

        monitor.start_monitoring().await;
    }

    pub async fn handle_request(&self, request: ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let domain = self.get_domain_for_request(&request)?;
        let proxy_builder = self.create_proxy_builder();

        if !self.retry_config.enabled || domain.urls.len() <= 1 {
            let Some(node_domain) = NodeService::get_node_domain(&self.nodes, domain.domain.clone()).await else {
                return self.create_error_response(&request, None, &format!("Node domain not found for domain: {}", domain.domain));
            };
            match proxy_builder.handle_request(request.clone(), &node_domain).await {
                Ok(response) if self.should_retry_response(&request, &response) => {
                    return self.create_error_response(&request, Some(&node_domain.url.host()), &format!("Upstream status code: {}", response.status));
                }
                result => return result,
            }
        }

        for url in &domain.urls {
            let node_domain = NodeDomain::new(url.clone(), domain.clone());
            if let Ok(response) = proxy_builder.handle_request(request.clone(), &node_domain).await
                && !self.should_retry_response(&request, &response)
            {
                return Ok(response);
            }
        }

        self.create_error_response(&request, None, "All upstream URLs failed")
    }

    pub(crate) fn get_domain_for_request(&self, request: &ProxyRequest) -> Result<&Domain, Box<dyn Error + Send + Sync>> {
        self.domains
            .get(&request.host)
            .or_else(|| {
                self.domains
                    .values()
                    .filter(|d| d.chain == request.chain)
                    .max_by_key(|d| d.domain.len())
            })
            .ok_or_else(|| format!("Domain for chain {} not found", request.chain).into())
    }

    fn should_retry_response(&self, request: &ProxyRequest, response: &ProxyResponse) -> bool {
        if self.retry_config.status_codes.contains(&response.status) {
            return true;
        }

        match request.request_type() {
            RequestType::JsonRpc(_) if response.status == StatusCode::OK.as_u16() && !self.retry_config.error_messages.is_empty() => {
                if let Ok(error_response) = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body) {
                    return self.retry_config.should_retry_on_error_message(&error_response.error.message);
                }
                false
            }
            _ => false,
        }
    }

    fn create_error_response(&self, request: &ProxyRequest, host: Option<&str>, error_message: &str) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let upstream_headers = ResponseBuilder::create_upstream_headers(host, request.elapsed());

        let response = match request.request_type() {
            RequestType::JsonRpc(_) => serde_json::to_value(JsonRpcErrorResponse::new("Internal error", Some(json!(error_message))))?,
            RequestType::Regular { .. } => serde_json::to_value(ResponseError {
                error: ErrorDetail {
                    message: error_message.to_string(),
                    data: None,
                },
            })?,
        };

        let body = serde_json::to_vec(&response)?;

        ResponseBuilder::build_with_headers(body, StatusCode::INTERNAL_SERVER_ERROR.as_u16(), JSON_CONTENT_TYPE, upstream_headers)
    }

    fn create_proxy_builder(&self) -> ProxyBuilder {
        ProxyBuilder::new(self.metrics.as_ref().clone(), self.cache.clone(), self.http_client.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CacheConfig, MetricsConfig, Url};
    use primitives::Chain;
    use reqwest::{Method, header::HeaderMap};

    fn create_service(domains: HashMap<String, Domain>) -> NodeService {
        NodeService::new(
            domains,
            Metrics::new(MetricsConfig::default()),
            CacheConfig::default(),
            NodeMonitoringConfig::default(),
            RetryConfig {
                enabled: false,
                status_codes: vec![],
                error_messages: vec![],
            },
            RequestConfig { timeout: 30000 },
        )
    }

    fn create_domain(domain: &str, chain: Chain, url: &str) -> Domain {
        Domain {
            domain: domain.to_string(),
            chain,
            block_delay: None,
            poll_interval_seconds: None,
            overrides: None,
            urls: vec![Url {
                url: url.to_string(),
                headers: None,
            }],
        }
    }

    fn create_request(host: &str, chain: Chain) -> ProxyRequest {
        ProxyRequest::new(Method::POST, HeaderMap::new(), vec![], "/".to_string(), "/".to_string(), host.to_string(), "test".to_string(), chain)
    }

    #[test]
    fn test_get_domain_for_request_exact_match() {
        let mut domains = HashMap::new();
        domains.insert("bitcoin".to_string(), create_domain("bitcoin", Chain::Bitcoin, "https://bitcoin.example.com"));
        let service = create_service(domains);
        let request = create_request("bitcoin", Chain::Bitcoin);

        let result = service.get_domain_for_request(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().domain, "bitcoin");
    }

    #[test]
    fn test_get_domain_for_request_chain_fallback_longest() {
        let mut domains = HashMap::new();
        domains.insert("bitcoin".to_string(), create_domain("bitcoin", Chain::Bitcoin, "https://bitcoin.example.com"));
        domains.insert("bitcoin.internal".to_string(), create_domain("bitcoin.internal", Chain::Bitcoin, "https://bitcoin-internal.example.com"));
        let service = create_service(domains);
        let request = create_request("unknown", Chain::Bitcoin);

        let result = service.get_domain_for_request(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().domain, "bitcoin.internal");
    }

    #[test]
    fn test_get_domain_for_request_not_found() {
        let mut domains = HashMap::new();
        domains.insert("bitcoin".to_string(), create_domain("bitcoin", Chain::Bitcoin, "https://bitcoin.example.com"));
        let service = create_service(domains);
        let request = create_request("unknown", Chain::Ethereum);

        let result = service.get_domain_for_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Domain for chain ethereum not found"));
    }

    #[test]
    fn test_get_domain_for_request_exact_match_priority() {
        let mut domains = HashMap::new();
        domains.insert("bitcoin".to_string(), create_domain("bitcoin", Chain::Bitcoin, "https://bitcoin.example.com"));
        domains.insert("bitcoin.internal".to_string(), create_domain("bitcoin.internal", Chain::Bitcoin, "https://bitcoin-internal.example.com"));
        let service = create_service(domains);
        let request = create_request("bitcoin", Chain::Bitcoin);

        let result = service.get_domain_for_request(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().domain, "bitcoin");
    }
}
