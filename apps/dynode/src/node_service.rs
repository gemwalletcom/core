use std::{collections::HashMap, sync::Arc, time::Instant};

use futures::future;
use gem_tracing::info_with_context;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::cache::RequestCache;
use crate::chain_client::ChainClient;
use crate::config::{CacheConfig, Url};
use crate::metrics::Metrics;
use crate::{
    config::Domain,
    proxy_request_service::{NodeDomain, ProxyRequestService},
};

#[derive(Debug, Clone)]
pub struct NodeService {
    pub domains: HashMap<String, Domain>,
    pub nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    pub cache: RequestCache,
}

#[derive(Debug)]
pub struct NodeRawResult {
    pub url: Url,
    pub result: Result<u64, Box<dyn std::error::Error + Send + Sync>>,
    pub latency: u64,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub url: Url,
    pub block_number: u64,
    pub latency: u64,
}

impl NodeService {
    pub fn new(domains: HashMap<String, Domain>, metrics: Metrics, cache_config: CacheConfig) -> Self {
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
        }
    }

    pub async fn get_proxy_request(&self) -> ProxyRequestService {
        ProxyRequestService {
            domains: self.get_node_domains().await,
            domain_configs: self.domains.clone(),
            metrics: self.metrics.as_ref().clone(),
            cache: self.cache.clone(),
        }
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

    pub async fn update_block_numbers(&self) {
        for (_, domain) in self.domains.clone() {
            self.metrics.set_node_host_current(&domain.domain, &domain.urls.first().unwrap().url);

            if domain.urls.len() > 1 {
                let domain = domain.clone();

                let nodes = Arc::clone(&self.nodes);

                tokio::task::spawn(async move {
                    loop {
                        let tasks: Vec<_> = domain
                            .clone()
                            .urls
                            .iter()
                            .map(|url| {
                                let chain = domain.chain.clone();
                                let url = url.clone();
                                tokio::spawn(async move {
                                    let now = Instant::now();
                                    let client = ChainClient::new(chain, url.url.clone());
                                    let result = client.get_latest_block().await;

                                    NodeRawResult {
                                        url: url.clone(),
                                        result,
                                        latency: now.elapsed().as_millis() as u64,
                                    }
                                })
                            })
                            .collect();

                        let results: Vec<NodeResult> = future::join_all(tasks)
                            .await
                            .into_iter()
                            .filter_map(|res| res.ok())
                            .filter_map(|res| {
                                res.result.ok().map(|block_number| NodeResult {
                                    url: res.url,
                                    block_number,
                                    latency: res.latency,
                                })
                            })
                            .collect();

                        if let Some(value) = Self::get_node_domain(&nodes.clone(), domain.domain.clone()).await {
                            let is_url_behind = domain.is_url_behind(value.url.clone(), results.clone());

                            let blocks_str = format!("{:?}", results.clone().into_iter().map(|x| x.block_number).collect::<Vec<u64>>());

                            info_with_context(
                                "Status",
                                &[
                                    ("url", &value.url.url),
                                    ("is_behind", &is_url_behind.to_string()),
                                    ("block_numbers", &blocks_str),
                                ],
                            );
                            if is_url_behind {
                                if let Some(node) = Domain::find_highest_block_number(results.clone()) {
                                    Self::update_node_domain(&nodes, domain.domain.clone(), NodeDomain { url: node.url.clone() }).await;

                                    info_with_context(
                                        "Node switch",
                                        &[
                                            ("domain", &domain.domain),
                                            ("new_node", &node.url.url),
                                            ("latency_ms", &node.latency.to_string()),
                                        ],
                                    );
                                }
                            }
                        }

                        sleep(Duration::from_secs(domain.get_poll_interval_seconds())).await;
                    }
                });
            }
        }
    }
}
