use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::cache::RequestCache;
use crate::config::{CacheConfig, Domain, NodeMonitoringConfig};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::proxy::{NodeDomain, ProxyRequestService};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
    pub nodes: Arc<RwLock<HashMap<String, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub cache: RequestCache,
    pub monitoring_config: NodeMonitoringConfig,
    pub http_client: reqwest::Client,
}

impl NodeService {
    pub fn new(domains: HashMap<String, Domain>, metrics: Metrics, cache_config: CacheConfig, monitoring_config: NodeMonitoringConfig) -> Self {
        let mut hash_map: HashMap<String, NodeDomain> = HashMap::new();

        for (key, domain) in domains.clone() {
            let url = domain.urls.first().unwrap().clone();
            hash_map.insert(key, NodeDomain { url });
        }

        let http_client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap();

        Self {
            domains,
            nodes: Arc::new(RwLock::new(hash_map)),
            metrics: Arc::new(metrics),
            cache: RequestCache::new(cache_config),
            monitoring_config,
            http_client,
        }
    }

    pub async fn get_proxy_request(&self) -> ProxyRequestService {
        let node_domains = self.nodes.read().await.clone();
        ProxyRequestService::new(
            node_domains,
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

    pub async fn start_monitoring(&self) {
        let monitor = NodeMonitor::new(
            self.domains.clone(),
            Arc::clone(&self.nodes),
            Arc::clone(&self.metrics),
            self.monitoring_config.clone(),
        );

        monitor.start_monitoring().await;
    }

    pub async fn handle_request_with_fallback(
        &self,
        method: reqwest::Method,
        headers: reqwest::header::HeaderMap,
        body_vec: Vec<u8>,
        path: String,
        path_with_query: String,
        host: String,
        user_agent: String,
    ) -> Result<crate::proxy::ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let domain = self.domains.get(&host);

        if domain.is_none() || domain.unwrap().urls.len() <= 1 {
            let proxy_service = self.get_proxy_request().await;
            return proxy_service
                .handle_request(method, headers, body_vec, path, path_with_query, host, user_agent)
                .await;
        }

        let domain = domain.unwrap();
        let current_node = Self::get_node_domain(&self.nodes, host.clone()).await;

        for url in &domain.urls {
            if let Some(ref current) = current_node {
                if url.url == current.url.url && url == domain.urls.first().unwrap() {
                } else if url.url == current.url.url {
                    continue;
                }
            }

            Self::update_node_domain(&self.nodes, host.clone(), NodeDomain { url: url.clone() }).await;

            let proxy_service = self.get_proxy_request().await;
            match proxy_service
                .handle_request(
                    method.clone(),
                    headers.clone(),
                    body_vec.clone(),
                    path.clone(),
                    path_with_query.clone(),
                    host.clone(),
                    user_agent.clone(),
                )
                .await
            {
                Ok(response) => {
                    if !Self::should_retry_with_fallback(response.status) {
                        return Ok(response);
                    }
                    // Continue to next URL if this one returns 429/403
                }
                Err(_) => {
                    // Continue to next URL on error
                    continue;
                }
            }
        }

        if let Some(original_node) = current_node {
            Self::update_node_domain(&self.nodes, host.clone(), original_node).await;
        }

        Err("All URLs failed".into())
    }

    fn should_retry_with_fallback(status: u16) -> bool {
        matches!(status, 429 | 403)
    }
}
