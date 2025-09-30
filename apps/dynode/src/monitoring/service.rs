use std::error::Error;
use std::{collections::HashMap, sync::Arc};

use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::cache::RequestCache;
use crate::config::{CacheConfig, Domain, NodeMonitoringConfig, RequestConfig, RetryConfig};
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_builder::ProxyBuilder;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::response_builder::ResponseBuilder;
use crate::proxy::{NodeDomain, ProxyRequestService, ProxyResponse};
use primitives::{ResponseError, response::ErrorDetail};
use serde_json::json;

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
            hash_map.insert(key, NodeDomain { url });
        }

        let http_client = gem_client::default_client_builder()
            .timeout(std::time::Duration::from_secs(request_config.timeout_seconds))
            .connect_timeout(std::time::Duration::from_secs(request_config.connect_timeout_seconds))
            .build()
            .unwrap();

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

    pub async fn get_proxy_request(&self) -> ProxyRequestService {
        ProxyRequestService::new(
            Arc::clone(&self.nodes),
            self.domains.clone(),
            self.metrics.as_ref().clone(),
            self.cache.clone(),
            self.http_client.clone(),
        )
    }

    pub async fn get_node_domain(nodes: &Arc<RwLock<HashMap<String, NodeDomain>>>, domain: String) -> Option<NodeDomain> {
        (nodes.read().await).get(&domain).cloned()
    }

    pub async fn update_node_domain(nodes: &Arc<RwLock<HashMap<String, NodeDomain>>>, domain: String, node_domain: NodeDomain) {
        let mut map = nodes.write().await;
        map.insert(domain, node_domain);
    }

    pub fn get_chain_for_host(&self, host: &str) -> Option<primitives::Chain> {
        self.domains.get(host).map(|domain| domain.chain)
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
        let Some(domain) = self.domains.get(&request.host) else {
            return self.create_error_response(&request, None, &format!("Domain {} not found", request.host));
        };

        if !self.retry_config.enabled || domain.urls.len() <= 1 {
            let proxy_service = self.get_proxy_request().await;
            let Some(node_domain) = NodeService::get_node_domain(&self.nodes, request.host.clone()).await else {
                return self.create_error_response(&request, None, &format!("Node domain not found for host: {}", request.host));
            };
            match proxy_service.handle_request(request.clone(), &node_domain).await {
                Ok(response) if self.retry_config.status_codes.contains(&response.status) => {
                    return self.create_error_response(&request, Some(&node_domain.url.host()), &format!("Upstream error: {}", response.status));
                }
                result => return result,
            }
        }

        for url in &domain.urls {
            if let Ok(response) = self.create_proxy_builder().handle_request_with_url(request.clone(), url).await
                && !self.retry_config.status_codes.contains(&response.status)
            {
                return Ok(response);
            }
        }

        self.create_error_response(&request, None, "All upstream URLs failed")
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
        ProxyBuilder::new(
            self.domains.clone(),
            self.metrics.as_ref().clone(),
            self.cache.clone(),
            self.http_client.clone(),
        )
    }
}
