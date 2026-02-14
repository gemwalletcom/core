use std::error::Error;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

use reqwest::StatusCode;
use tokio::sync::RwLock;

use super::request_health::RequestAdaptiveMonitor;
use super::switch_reason::NodeSwitchReason;
use crate::cache::RequestCache;
use crate::config::{CacheConfig, ChainConfig, ErrorMatcherConfig, HeadersConfig, NodeMonitoringConfig, RequestConfig, RetryConfig, Url};
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_builder::ProxyBuilder;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::response_builder::ResponseBuilder;
use crate::proxy::{NodeDomain, ProxyResponse};
use gem_tracing::{DurationMs, info_with_fields};
use primitives::{Chain, ResponseError, response::ErrorDetail};

const NODE_NOT_FOUND: &str = "Node not found";

#[derive(Debug, Clone)]
pub struct NodeService {
    pub chains: HashMap<Chain, ChainConfig>,
    pub nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub monitoring_config: NodeMonitoringConfig,
    pub retry_config: RetryConfig,
    request_adaptive_monitor: Arc<RequestAdaptiveMonitor>,
    proxy_builder: ProxyBuilder,
}

impl NodeService {
    pub fn new(
        chains: HashMap<Chain, ChainConfig>,
        metrics: Metrics,
        cache_config: CacheConfig,
        monitoring_config: NodeMonitoringConfig,
        retry_config: RetryConfig,
        request_config: RequestConfig,
        headers_config: HeadersConfig,
    ) -> Self {
        let nodes = chains.values().map(|c| (c.chain, NodeDomain::new(c.urls.first().unwrap().clone(), c.clone()))).collect();

        let http_client = gem_client::builder().timeout(request_config.timeout).build().unwrap();
        let cache = RequestCache::new(cache_config);
        let proxy_builder = ProxyBuilder::new(metrics.clone(), cache, http_client, headers_config);
        let request_adaptive_monitor = Arc::new(RequestAdaptiveMonitor::new(monitoring_config.adaptive.clone()));

        Self {
            chains,
            nodes: Arc::new(RwLock::new(nodes)),
            metrics: Arc::new(metrics),
            monitoring_config,
            retry_config,
            request_adaptive_monitor,
            proxy_builder,
        }
    }

    pub async fn get_node_domain(nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, chain: Chain) -> Option<NodeDomain> {
        nodes.read().await.get(&chain).cloned()
    }

    pub fn sync_current_node_metric(metrics: &Arc<Metrics>, chain: Chain, url: &Url) {
        metrics.set_node_host_current(chain.as_ref(), &url.host());
    }

    pub async fn switch_node_if_current(
        nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: &Arc<Metrics>,
        chain_config: &ChainConfig,
        expected_current: &Url,
        selected: &Url,
        reason: &NodeSwitchReason,
    ) -> Option<(String, String)> {
        let (old_host, new_host) = {
            let mut nodes_write = nodes.write().await;
            let active_node = nodes_write.get(&chain_config.chain)?;
            if active_node.url.url != expected_current.url || active_node.url.url == selected.url {
                return None;
            }

            let old_host = active_node.url.host();
            let new_host = selected.host();
            nodes_write.insert(chain_config.chain, NodeDomain::new(selected.clone(), chain_config.clone()));
            (old_host, new_host)
        };

        Self::sync_current_node_metric(metrics, chain_config.chain, selected);
        metrics.add_node_switch(chain_config.chain.as_ref(), &old_host, &new_host, reason.as_str(), reason.metric_detail());
        Some((old_host, new_host))
    }

    pub async fn handle_request(&self, request: ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let chain_config = self.get_chain_config(&request)?;
        let Some(urls) = self.resolve_request_urls(chain_config, &request).await else {
            return self.node_not_found_response(&request);
        };
        if urls.len() == 1 {
            let primary = NodeDomain::new(urls[0].clone(), chain_config.clone());
            return self.proxy_builder.handle_request(request, &primary).await;
        }

        let retry_enabled = self.retry_config.enabled && urls.len() > 1;
        let mut last_error: Option<String> = None;
        let max_attempts = if retry_enabled { self.retry_config.effective_max_attempts(urls.len()) } else { 1 };

        for url in urls.iter().take(max_attempts) {
            let node_domain = NodeDomain::new(url.clone(), chain_config.clone());
            let remote_host = url.host();
            match self.proxy_builder.handle_request(request.clone(), &node_domain).await {
                Ok(response) => {
                    let retry_error = self.matches_response_error_signal(&request, &response, &self.retry_config.errors);
                    let adaptive_error = self.matches_response_error_signal(&request, &response, &self.monitoring_config.adaptive.errors);
                    self.record_attempt(chain_config.chain, remote_host.as_str(), adaptive_error).await;
                    if !retry_error {
                        self.switch_after_success_if_allowed(chain_config, url).await;
                        return Ok(response);
                    }

                    if !retry_enabled {
                        return self.log_and_create_error_response(&request, Some(remote_host.as_str()), &format!("Upstream status code: {}", response.status));
                    }
                    last_error = Some(format!("status={}", response.status));
                }
                Err(e) => {
                    self.record_attempt(chain_config.chain, remote_host.as_str(), true).await;
                    if !retry_enabled {
                        return Err(e);
                    }

                    let error = e.to_string();
                    let request_id = request.id.as_str();
                    let chain = request.chain.as_ref();
                    let user_agent = request.user_agent.as_str();
                    let latency = DurationMs(request.elapsed());
                    info_with_fields!(
                        "Upstream error",
                        id = request_id,
                        chain = chain,
                        remote_host = remote_host.as_str(),
                        error = error.as_str(),
                        user_agent = user_agent,
                        latency = latency,
                    );
                    last_error = Some(error);
                }
            }
        }

        let error_message = last_error
            .map(|e| format!("All upstream URLs failed, {}", e))
            .unwrap_or_else(|| "All upstream URLs failed".to_string());
        self.log_and_create_error_response(&request, None, &error_message)
    }

    fn get_chain_config(&self, request: &ProxyRequest) -> Result<&ChainConfig, Box<dyn Error + Send + Sync>> {
        self.chains.get(&request.chain).ok_or_else(|| format!("Chain {} not configured", request.chain).into())
    }

    async fn resolve_request_urls(&self, chain_config: &ChainConfig, request: &ProxyRequest) -> Option<Vec<Url>> {
        if chain_config.urls.is_empty() {
            return None;
        }
        if chain_config.urls.len() == 1 {
            return Some(vec![chain_config.urls[0].clone()]);
        }

        let current_node = NodeService::get_node_domain(&self.nodes, chain_config.chain).await?;
        let urls = self.get_ordered_urls(chain_config.chain, &chain_config.urls, &current_node.url, request.id.as_str()).await;
        if urls.is_empty() {
            return None;
        }

        Some(urls)
    }

    fn node_not_found_response(&self, request: &ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        self.log_and_create_error_response(request, None, NODE_NOT_FOUND)
    }

    async fn get_ordered_urls(&self, chain: Chain, urls: &[Url], current: &Url, request_id: &str) -> Vec<Url> {
        let mut ordered_urls = urls.to_vec();
        if let Some(current_index) = ordered_urls.iter().position(|url| *url == *current) {
            ordered_urls.swap(0, current_index);
        }

        Self::rotate_fallback_urls(&mut ordered_urls, request_id);

        if ordered_urls.len() <= 1 || !self.request_adaptive_monitor.is_enabled() {
            return ordered_urls;
        }

        self.request_adaptive_monitor.reorder_urls(chain, &ordered_urls).await
    }

    fn rotate_fallback_urls(urls: &mut [Url], request_id: &str) {
        if urls.len() <= 2 {
            return;
        }

        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        let tail_len = urls.len() - 1;
        let offset = (hasher.finish() as usize) % tail_len;
        if offset > 0 {
            urls[1..].rotate_left(offset);
        }
    }

    async fn record_attempt(&self, chain: Chain, host: &str, has_error_signal: bool) {
        let snapshot = self.request_adaptive_monitor.record_attempt(chain, host, has_error_signal).await;
        let Some(snapshot) = snapshot else {
            return;
        };

        if !snapshot.blocked_now {
            return;
        }

        let ratio = format!("{:.3}", snapshot.ratio);
        info_with_fields!(
            "Node host blocked",
            chain = chain.as_ref(),
            host = host,
            error_ratio = ratio.as_str(),
            samples = snapshot.total,
            errors = snapshot.errors,
        );
    }

    async fn switch_after_success_if_allowed(&self, chain_config: &ChainConfig, selected_url: &Url) {
        let Some(current_node) = NodeService::get_node_domain(&self.nodes, chain_config.chain).await else {
            return;
        };
        if current_node.url.url == selected_url.url {
            return;
        }

        let old_host = current_node.url.host();
        let new_host = selected_url.host();
        let snapshot = self
            .request_adaptive_monitor
            .allow_switch_after_success(chain_config.chain, old_host.as_str(), new_host.as_str())
            .await;
        let Some(snapshot) = snapshot else {
            return;
        };

        let ratio = if snapshot.ratio.is_finite() { snapshot.ratio.clamp(0.0, 1.0) } else { 1.0 };
        let reason = NodeSwitchReason::AdaptiveError {
            error_ratio: ratio,
            samples: snapshot.total,
        };
        let detail = reason.to_string();
        let Some((old_host, new_host)) = NodeService::switch_node_if_current(&self.nodes, &self.metrics, chain_config, &current_node.url, selected_url, &reason).await else {
            return;
        };
        self.request_adaptive_monitor.mark_switch(chain_config.chain).await;

        info_with_fields!(
            "Node switch",
            chain = chain_config.chain.as_ref(),
            old_host = old_host.as_str(),
            new_host = new_host.as_str(),
            reason = reason.as_str(),
            detail = detail.as_str(),
        );
    }

    fn matches_response_error_signal(&self, request: &ProxyRequest, response: &ProxyResponse, matcher: &ErrorMatcherConfig) -> bool {
        if matcher.matches_status(response.status) {
            return true;
        }

        match request.request_type() {
            RequestType::JsonRpc(_) if response.status == StatusCode::OK.as_u16() => {
                if let Ok(error_response) = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body) {
                    return matcher.matches_message(&error_response.error.message);
                }
                false
            }
            _ => false,
        }
    }

    fn log_and_create_error_response(&self, request: &ProxyRequest, host: Option<&str>, error_message: &str) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let request_id = request.id.as_str();
        let chain = request.chain.as_ref();
        let uri = request.path.as_str();
        let method = request.method.as_str();
        let remote_host = host.unwrap_or("none");
        let user_agent = request.user_agent.as_str();
        let latency = DurationMs(request.elapsed());
        let status: u16 = 500;
        info_with_fields!(
            "Proxy response",
            id = request_id,
            chain = chain,
            remote_host = remote_host,
            method = method,
            uri = uri,
            status = status,
            error = error_message,
            user_agent = user_agent,
            latency = latency,
        );

        let upstream_headers = ResponseBuilder::create_upstream_headers(host, request.elapsed());

        let response = match request.request_type() {
            RequestType::JsonRpc(_) => serde_json::to_value(JsonRpcErrorResponse::new(error_message))?,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CacheConfig, MetricsConfig, Url};
    use crate::testkit::config as testkit;
    use primitives::Chain;
    use reqwest::{Method, header, header::HeaderMap};

    fn create_service(chains: HashMap<Chain, ChainConfig>) -> NodeService {
        create_service_with_retry(chains, create_retry_config(false, vec![], vec![]))
    }

    fn create_service_with_retry(chains: HashMap<Chain, ChainConfig>, retry_config: RetryConfig) -> NodeService {
        NodeService::new(
            chains,
            Metrics::new(MetricsConfig::default()),
            CacheConfig::default(),
            testkit::monitoring_config(),
            retry_config,
            RequestConfig {
                timeout: std::time::Duration::from_millis(30000),
            },
            HeadersConfig {
                forward: vec![header::CONTENT_TYPE.to_string()],
                domains: HashMap::new(),
            },
        )
    }

    fn create_retry_config(enabled: bool, status_codes: Vec<u16>, error_messages: Vec<&str>) -> RetryConfig {
        testkit::retry_config(enabled, status_codes, error_messages)
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

    #[test]
    fn test_adaptive_uses_retry_status_codes() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "https://ethereum.example.com"))]);
        let service = create_service_with_retry(chains, create_retry_config(true, vec![429], vec![]));

        let request = create_request("ethereum.example.com", Chain::Ethereum);
        let response = ProxyResponse::new(429, HeaderMap::new(), vec![]);

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }

    #[test]
    fn test_adaptive_uses_retry_jsonrpc_messages() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "https://ethereum.example.com"))]);
        let service = create_service_with_retry(chains, create_retry_config(true, vec![], vec!["Exceeded the quota usage"]));

        let request = ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#.to_vec(),
            "/".to_string(),
            "/".to_string(),
            "ethereum.example.com".to_string(),
            "test".to_string(),
            Chain::Ethereum,
        );
        let response = ProxyResponse::new(
            200,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Exceeded the quota usage"},"id":1}"#.to_vec(),
        );

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }
}
