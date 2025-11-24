use std::{collections::HashMap, sync::Arc};

use futures::future;
use primitives::Chain;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

use super::chain_client::ChainClient;
use super::sync::{NodeStatusObservation, NodeSyncAnalyzer};
use super::telemetry::NodeTelemetry;
use crate::config::{ChainConfig, NodeMonitoringConfig, Url};
use crate::metrics::Metrics;
use crate::monitoring::NodeService;
use crate::proxy::NodeDomain;

pub struct NodeMonitor {
    chains: HashMap<Chain, ChainConfig>,
    nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    metrics: Arc<Metrics>,
    monitoring_config: NodeMonitoringConfig,
}

impl NodeMonitor {
    pub fn new(
        chains: HashMap<Chain, ChainConfig>,
        nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: Arc<Metrics>,
        monitoring_config: NodeMonitoringConfig,
    ) -> Self {
        Self {
            chains,
            nodes,
            metrics,
            monitoring_config,
        }
    }

    pub async fn start_monitoring(&self) {
        for (index, chain_config) in self.chains.values().cloned().enumerate() {
            if chain_config.urls.len() <= 1 {
                if let Some(url) = chain_config.urls.first() {
                    self.metrics.set_node_host_current(chain_config.chain.as_ref(), &url.url);
                }
                continue;
            }

            if let Some(url) = chain_config.urls.first() {
                self.metrics.set_node_host_current(chain_config.chain.as_ref(), &url.url);
            }

            let nodes = Arc::clone(&self.nodes);
            let metrics = Arc::clone(&self.metrics);
            let monitoring_config = self.monitoring_config.clone();
            let initial_delay = Duration::from_millis(((index as u64) + 1) * 250);

            tokio::task::spawn(async move {
                sleep(initial_delay).await;

                loop {
                    if let Err(err) = Self::evaluate_chain(&chain_config, &nodes, &metrics).await {
                        NodeTelemetry::log_monitor_error(&chain_config, err.as_ref());
                    }

                    sleep(Duration::from_secs(chain_config.get_poll_interval_seconds(&monitoring_config))).await;
                }
            });
        }
    }

    async fn evaluate_chain(
        chain_config: &ChainConfig,
        nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: &Arc<Metrics>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if chain_config.urls.len() <= 1 {
            return Ok(());
        }

        let current_node = match NodeService::get_node_domain(nodes, chain_config.chain).await {
            Some(node) => node,
            None => {
                NodeTelemetry::log_missing_current(chain_config);
                return Ok(());
            }
        };

        let current_observation = Self::fetch_status(chain_config.chain, current_node.url.clone()).await;
        NodeTelemetry::log_status_debug(chain_config, std::slice::from_ref(&current_observation));

        if NodeSyncAnalyzer::is_node_healthy(&current_observation) {
            NodeTelemetry::log_node_healthy(chain_config, &current_observation);
            return Ok(());
        }

        NodeTelemetry::log_node_unhealthy(chain_config, &current_observation);

        let fallback_urls: Vec<Url> = chain_config.urls.iter().filter(|&url| *url != current_node.url).cloned().collect();

        if fallback_urls.is_empty() {
            NodeTelemetry::log_no_candidate(chain_config, &[]);
            return Ok(());
        }

        let fallback_statuses = Self::fetch_statuses(chain_config.chain, fallback_urls).await;
        NodeTelemetry::log_status_debug(chain_config, &fallback_statuses);

        if let Some(best_candidate) = NodeSyncAnalyzer::select_best_node(&current_node.url, &fallback_statuses) {
            if best_candidate.url.url != current_node.url.url {
                NodeService::update_node_domain(nodes, chain_config.chain, NodeDomain::new(best_candidate.url.clone(), chain_config.clone())).await;
                metrics.set_node_host_current(chain_config.chain.as_ref(), &best_candidate.url.url);
                metrics.add_node_switch(chain_config.chain.as_ref(), &current_node.url.url, &best_candidate.url.url);

                NodeTelemetry::log_node_switch(chain_config, &current_node.url, &best_candidate);
            }
        } else {
            NodeTelemetry::log_no_candidate(chain_config, &fallback_statuses);
        }

        Ok(())
    }

    async fn fetch_statuses(chain: Chain, urls: Vec<Url>) -> Vec<NodeStatusObservation> {
        let futures = urls.into_iter().map(move |url| Self::fetch_status(chain, url));

        future::join_all(futures).await
    }

    async fn fetch_status(chain: Chain, url: Url) -> NodeStatusObservation {
        let client = ChainClient::new(chain, url);
        client.fetch_status().await
    }
}
