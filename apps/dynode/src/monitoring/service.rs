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
}
