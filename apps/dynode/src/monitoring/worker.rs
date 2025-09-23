use std::{collections::HashMap, sync::Arc};

use futures::future;
use primitives::Chain;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use super::chain_client::ChainClient;
use super::sync::{NodeStatusObservation, NodeSyncAnalyzer};
use super::telemetry::NodeTelemetry;
use crate::config::{Domain, NodeMonitoringConfig, Url};
use crate::metrics::Metrics;
use crate::monitoring::NodeService;
use crate::proxy::NodeDomain;

pub struct NodeMonitor {
    domains: HashMap<String, Domain>,
    nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
    metrics: Arc<Metrics>,
    monitoring_config: NodeMonitoringConfig,
}

impl NodeMonitor {
    pub fn new(
        domains: HashMap<String, Domain>,
        nodes: Arc<Mutex<HashMap<String, NodeDomain>>>,
        metrics: Arc<Metrics>,
        monitoring_config: NodeMonitoringConfig,
    ) -> Self {
        Self {
            domains,
            nodes,
            metrics,
            monitoring_config,
        }
    }

    pub async fn start_monitoring(&self) {
        for (index, domain) in self.domains.values().cloned().enumerate() {
            if domain.urls.len() <= 1 {
                if let Some(url) = domain.urls.first() {
                    self.metrics.set_node_host_current(&domain.domain, &url.url);
                }
                continue;
            }

            if let Some(url) = domain.urls.first() {
                self.metrics.set_node_host_current(&domain.domain, &url.url);
            }

            let domain_clone = domain;
            let nodes = Arc::clone(&self.nodes);
            let metrics = Arc::clone(&self.metrics);
            let monitoring_config = self.monitoring_config.clone();
            let initial_delay = Duration::from_millis(((index as u64) + 1) * 250);

            tokio::task::spawn(async move {
                sleep(initial_delay).await;

                loop {
                    if let Err(err) = Self::evaluate_domain(&domain_clone, &nodes, &metrics).await {
                        NodeTelemetry::log_monitor_error(&domain_clone, err.as_ref());
                    }

                    sleep(Duration::from_secs(domain_clone.get_poll_interval_seconds(&monitoring_config))).await;
                }
            });
        }
    }

    async fn evaluate_domain(
        domain: &Domain,
        nodes: &Arc<Mutex<HashMap<String, NodeDomain>>>,
        metrics: &Arc<Metrics>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if domain.urls.len() <= 1 {
            return Ok(());
        }

        let current_node = match NodeService::get_node_domain(nodes, domain.domain.clone()).await {
            Some(node) => node,
            None => {
                NodeTelemetry::log_missing_current(domain);
                return Ok(());
            }
        };

        let current_observation = Self::fetch_status(domain.chain, current_node.url.clone()).await;
        NodeTelemetry::log_status_debug(domain, std::slice::from_ref(&current_observation));

        if NodeSyncAnalyzer::is_node_healthy(&current_observation) {
            NodeTelemetry::log_node_healthy(domain, &current_observation);
            return Ok(());
        }

        NodeTelemetry::log_node_unhealthy(domain, &current_observation);

        let fallback_urls: Vec<Url> = domain.urls.iter().filter(|&url| *url != current_node.url).cloned().collect();

        if fallback_urls.is_empty() {
            NodeTelemetry::log_no_candidate(domain, &[]);
            return Ok(());
        }

        let fallback_statuses = Self::fetch_statuses(domain.chain, fallback_urls).await;
        NodeTelemetry::log_status_debug(domain, &fallback_statuses);

        if let Some(best_candidate) = NodeSyncAnalyzer::select_best_node(&current_node.url, &fallback_statuses) {
            if best_candidate.url.url != current_node.url.url {
                NodeService::update_node_domain(
                    nodes,
                    domain.domain.clone(),
                    NodeDomain {
                        url: best_candidate.url.clone(),
                    },
                )
                .await;
                metrics.set_node_host_current(&domain.domain, &best_candidate.url.url);
                metrics.add_node_switch(domain.chain.as_ref(), &current_node.url.url, &best_candidate.url.url);

                NodeTelemetry::log_node_switch(domain, &current_node.url, &best_candidate);
            }
        } else {
            NodeTelemetry::log_no_candidate(domain, &fallback_statuses);
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
