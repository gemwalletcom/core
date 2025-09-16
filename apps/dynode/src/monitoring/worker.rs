use std::{collections::HashMap, sync::Arc, time::Instant};

use futures::future;
use gem_tracing::{error_with_fields, info_with_fields};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::metrics::Metrics;
use crate::monitoring::NodeService;
use crate::config::{Domain, NodeResult, Url};
use crate::proxy::NodeDomain;
use super::chain_client::ChainClient;

pub struct NodeMonitor {
    domains: HashMap<String, Domain>,
    nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
    metrics: Arc<Metrics>,
}

impl NodeMonitor {
    pub fn new(domains: HashMap<String, Domain>, nodes: Arc<Mutex<HashMap<String, NodeDomain>>>, metrics: Arc<Metrics>) -> Self {
        Self { domains, nodes, metrics }
    }

    pub async fn start_monitoring(&self) {
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
                                let chain = domain.chain;
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

                        if let Some(value) = NodeService::get_node_domain(&nodes.clone(), domain.domain.clone()).await {
                            let is_url_behind = domain.is_url_behind(value.url.clone(), results.clone());

                            let blocks_str = format!(
                                "{:?}",
                                results
                                    .clone()
                                    .into_iter()
                                    .map(|x| x.block_number)
                                    .collect::<Vec<u64>>()
                            );

                            if is_url_behind {
                                error_with_fields!(
                                    "Status",
                                    &std::io::Error::other("Node behind"),
                                    domain = &domain.domain,
                                    url = &value.url.url,
                                    blocks = &blocks_str,
                                );
                            } else {
                                info_with_fields!("Status", url = &value.url.url, is_behind = &is_url_behind.to_string(), blocks = &blocks_str,);
                            }

                            if is_url_behind {
                                if let Some(node) = Domain::find_highest_block_number(results.clone()) {
                                    NodeService::update_node_domain(&nodes, domain.domain.clone(), NodeDomain { url: node.url.clone() }).await;

                                    info_with_fields!(
                                        "Node switch",
                                        domain = &domain.domain,
                                        new_node = &node.url.url,
                                        latency_ms = &node.latency.to_string(),
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

    pub async fn check_nodes_sync(&self, _domain: &str) -> bool {
        true
    }

    pub async fn get_node_health(&self, _domain: &str) -> NodeHealth {
        NodeHealth::Healthy
    }
}

#[derive(Debug)]
pub struct NodeRawResult {
    pub url: Url,
    pub result: Result<u64, Box<dyn std::error::Error + Send + Sync>>,
    pub latency: u64,
}

#[derive(Debug, Clone)]
pub enum NodeHealth {
    Healthy,
    Behind,
    Unreachable,
}
