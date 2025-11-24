use std::error::Error;
use std::{collections::HashMap, sync::Arc};

use reqwest::StatusCode;
use tokio::sync::RwLock;

use crate::cache::RequestCache;
use crate::config::{CacheConfig, ChainConfig, NodeMonitoringConfig, RequestConfig, RetryConfig};
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_builder::ProxyBuilder;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::response_builder::ResponseBuilder;
use crate::proxy::{NodeDomain, ProxyResponse};
use primitives::{Chain, ResponseError, response::ErrorDetail};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NodeService {
    pub chains: HashMap<Chain, ChainConfig>,
    pub nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub cache: RequestCache,
    pub monitoring_config: NodeMonitoringConfig,
    pub retry_config: RetryConfig,
    pub request_config: RequestConfig,
    pub http_client: reqwest::Client,
}

impl NodeService {
    pub fn new(
        chains: HashMap<Chain, ChainConfig>,
        metrics: Metrics,
        cache_config: CacheConfig,
        monitoring_config: NodeMonitoringConfig,
        retry_config: RetryConfig,
        request_config: RequestConfig,
    ) -> Self {
        let nodes = chains
            .values()
            .map(|c| (c.chain, NodeDomain::new(c.urls.first().unwrap().clone(), c.clone())))
            .collect();

        let http_client = gem_client::builder().timeout(Duration::from_millis(request_config.timeout)).build().unwrap();

        Self {
            chains,
            nodes: Arc::new(RwLock::new(nodes)),
            metrics: Arc::new(metrics),
            cache: RequestCache::new(cache_config),
            monitoring_config,
            retry_config,
            request_config,
            http_client,
        }
    }

    pub async fn get_node_domain(nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, chain: Chain) -> Option<NodeDomain> {
        nodes.read().await.get(&chain).cloned()
    }

    pub async fn update_node_domain(nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, chain: Chain, node_domain: NodeDomain) {
        nodes.write().await.insert(chain, node_domain);
    }

    pub async fn handle_request(&self, request: ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let chain_config = self.get_chain_config(&request)?;
        let urls = &chain_config.urls;
        let proxy_builder = self.create_proxy_builder();

        if !self.retry_config.enabled || urls.len() <= 1 {
            let Some(node_domain) = NodeService::get_node_domain(&self.nodes, chain_config.chain).await else {
                return self.create_error_response(&request, None, "Node not found");
            };
            let active_node = if let Some(url) = urls.first() {
                NodeDomain::new(url.clone(), chain_config.clone())
            } else {
                node_domain
            };
            match proxy_builder.handle_request(request.clone(), &active_node).await {
                Ok(response) if self.should_retry_response(&request, &response) => {
                    return self.create_error_response(&request, Some(&active_node.url.host()), &format!("Upstream status code: {}", response.status));
                }
                result => return result,
            }
        }

        for url in urls {
            let node_domain = NodeDomain::new(url.clone(), chain_config.clone());
            if let Ok(response) = proxy_builder.handle_request(request.clone(), &node_domain).await
                && !self.should_retry_response(&request, &response)
            {
                return Ok(response);
            }
        }

        self.create_error_response(&request, None, "All upstream URLs failed")
    }

    fn get_chain_config(&self, request: &ProxyRequest) -> Result<&ChainConfig, Box<dyn Error + Send + Sync>> {
        self.chains
            .get(&request.chain)
            .ok_or_else(|| format!("Chain {} not configured", request.chain).into())
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

    fn create_service(chains: HashMap<Chain, ChainConfig>) -> NodeService {
        NodeService::new(
            chains,
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

    fn create_chain_config(chain: Chain, url: &str) -> ChainConfig {
        ChainConfig {
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
        ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            vec![],
            "/".to_string(),
            "/".to_string(),
            host.to_string(),
            "test".to_string(),
            chain,
        )
    }

    #[test]
    fn test_get_chain_config_found() {
        let chains = HashMap::from([(Chain::Bitcoin, create_chain_config(Chain::Bitcoin, "https://bitcoin.example.com"))]);
        let service = create_service(chains);
        let request = create_request("any.host.com", Chain::Bitcoin);

        let result = service.get_chain_config(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().chain, Chain::Bitcoin);
    }

    #[test]
    fn test_get_chain_config_not_found() {
        let chains = HashMap::from([(Chain::Bitcoin, create_chain_config(Chain::Bitcoin, "https://bitcoin.example.com"))]);
        let service = create_service(chains);
        let request = create_request("unknown", Chain::Ethereum);

        let result = service.get_chain_config(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Chain ethereum not configured"));
    }
}
