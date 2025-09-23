use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::cache::RequestCache;
use crate::config::{CacheConfig, Domain, NodeMonitoringConfig};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::proxy::{NodeDomain, ProxyRequestService};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
    pub nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub cache: RequestCache,
    pub monitoring_config: NodeMonitoringConfig,
}

impl NodeService {
    pub fn new(domains: HashMap<String, Domain>, metrics: Metrics, cache_config: CacheConfig, monitoring_config: NodeMonitoringConfig) -> Self {
        let mut hash_map: HashMap<String, NodeDomain> = HashMap::new();

        for (key, domain) in domains.clone() {
            let url = domain.urls.first().unwrap().clone();
            hash_map.insert(key, NodeDomain { url });
        }

        Self {
            domains,
            nodes: Arc::new(Mutex::new(hash_map)),
            metrics: Arc::new(metrics),
            cache: RequestCache::new(cache_config),
            monitoring_config,
        }
    }

    pub async fn get_proxy_request(&self) -> ProxyRequestService {
        ProxyRequestService::new(
            self.get_node_domains().await,
            self.domains.clone(),
            self.metrics.as_ref().clone(),
            self.cache.clone(),
        )
    }

    pub async fn get_node_domain(nodes: &Arc<Mutex<HashMap<String, NodeDomain>>>, domain: String) -> Option<NodeDomain> {
        (nodes.lock().await).get(&domain).cloned()
    }

    pub async fn update_node_domain(nodes: &Arc<Mutex<HashMap<String, NodeDomain>>>, domain: String, node_domain: NodeDomain) {
        let mut map = nodes.lock().await;
        map.insert(domain, node_domain);
    }

    pub async fn get_node_domains(&self) -> HashMap<String, NodeDomain> {
        (*self.nodes.lock().await).clone()
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
}
