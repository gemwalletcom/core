use std::time::Duration;

use super::switch_reason::NodeSwitchReason;
use crate::config::{NodeMonitoringConfig, Url};
use primitives::{Chain, NodeStatusState, NodeSyncStatus};

#[derive(Debug, Clone)]
pub struct NodeStatusObservation {
    pub url: Url,
    pub state: NodeStatusState,
    pub latency: Duration,
}

impl NodeStatusObservation {
    pub fn new(url: Url, state: NodeStatusState, latency: Duration) -> Self {
        Self { url, state, latency }
    }
}

#[derive(Debug, Clone)]
pub struct NodeSwitchResult {
    pub observation: NodeStatusObservation,
    pub reason: NodeSwitchReason,
}

pub struct NodeSyncAnalyzer;

impl NodeSyncAnalyzer {
    pub fn is_node_healthy(observation: &NodeStatusObservation) -> bool {
        match &observation.state {
            NodeStatusState::Healthy(status) => status.in_sync,
            NodeStatusState::Error { .. } => false,
        }
    }

    pub fn select_best_node(current: &Url, observations: &[NodeStatusObservation], monitoring_config: &NodeMonitoringConfig, chain: Chain) -> Option<NodeSwitchResult> {
        let current_observation = observations.iter().find(|o| o.url == *current)?;

        let error_reason = match &current_observation.state {
            NodeStatusState::Error { message } => Some(NodeSwitchReason::CurrentNodeError { message: message.clone() }),
            NodeStatusState::Healthy(_) => None,
        };

        let (candidate, candidate_status) = Self::find_best_candidate(current, observations)?;

        if let Some(reason) = error_reason {
            return Some(NodeSwitchResult {
                observation: candidate.clone(),
                reason,
            });
        }

        let current_status = current_observation.state.as_status()?;
        let reason = Self::evaluate_switch(current_status, candidate_status, current_observation.latency, candidate.latency, monitoring_config, chain)?;

        Some(NodeSwitchResult {
            observation: candidate.clone(),
            reason,
        })
    }

    fn find_best_candidate<'a>(current: &Url, observations: &'a [NodeStatusObservation]) -> Option<(&'a NodeStatusObservation, &'a NodeSyncStatus)> {
        observations
            .iter()
            .filter(|observation| observation.url != *current)
            .filter_map(|observation| match observation.state.as_status() {
                Some(status) if status.in_sync => Some((observation, status)),
                _ => None,
            })
            .max_by(|(left_observation, left_status), (right_observation, right_status)| Self::compare_candidates(left_observation, left_status, right_observation, right_status))
    }

    fn evaluate_switch(
        current: &NodeSyncStatus,
        new: &NodeSyncStatus,
        current_latency: Duration,
        new_latency: Duration,
        monitoring_config: &NodeMonitoringConfig,
        chain: Chain,
    ) -> Option<NodeSwitchReason> {
        let old_block = Self::status_height(current);
        let new_block = Self::status_height(new);
        let block_delay_threshold = monitoring_config.block_delay_threshold(chain);

        if new_block > old_block + block_delay_threshold {
            return Some(NodeSwitchReason::BlockHeight { old_block, new_block });
        }

        if current.in_sync && !monitoring_config.is_latency_improvement_significant(current_latency, new_latency) {
            return None;
        }

        Some(NodeSwitchReason::Latency {
            old_latency_ms: current_latency.as_millis() as u64,
            new_latency_ms: new_latency.as_millis() as u64,
        })
    }

    pub fn format_status_summary(observations: &[NodeStatusObservation]) -> String {
        observations
            .iter()
            .map(|observation| match &observation.state {
                NodeStatusState::Healthy(status) => format!(
                    "{}:in_sync={} latest={} current={} latency={}ms",
                    observation.url.url,
                    status.in_sync,
                    Self::format_optional_number(status.latest_block_number),
                    Self::format_optional_number(status.current_block_number),
                    observation.latency.as_millis()
                ),
                NodeStatusState::Error { message } => format!("{}:error={} latency={}ms", observation.url.url, message, observation.latency.as_millis()),
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn format_optional_number(value: Option<u64>) -> String {
        value.map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string())
    }

    fn compare_candidates(
        left_observation: &NodeStatusObservation,
        left_status: &NodeSyncStatus,
        right_observation: &NodeStatusObservation,
        right_status: &NodeSyncStatus,
    ) -> std::cmp::Ordering {
        let left_height = Self::status_height(left_status);
        let right_height = Self::status_height(right_status);

        match left_height.cmp(&right_height) {
            std::cmp::Ordering::Equal => right_observation.latency.cmp(&left_observation.latency),
            other => other,
        }
    }

    fn status_height(status: &NodeSyncStatus) -> u64 {
        status.current_block_number.or(status.latest_block_number).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::config as testkit;
    use crate::testkit::sync::{healthy_observation, not_in_sync_observation, url};

    fn config() -> NodeMonitoringConfig {
        testkit::monitoring_config()
    }

    #[test]
    fn is_node_healthy_requires_in_sync() {
        let synced = healthy_observation("https://a", Some(100), Some(100), 10);
        let not_synced = not_in_sync_observation("https://b", Some(100), Some(90), 10);
        let error = NodeStatusObservation::new(url("https://c"), NodeStatusState::error("fail"), Duration::from_millis(10));

        assert!(NodeSyncAnalyzer::is_node_healthy(&synced));
        assert!(!NodeSyncAnalyzer::is_node_healthy(&not_synced));
        assert!(!NodeSyncAnalyzer::is_node_healthy(&error));
    }

    #[test]
    fn selects_highest_block_number() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            healthy_observation("https://b", Some(120), Some(120), 30),
            healthy_observation("https://c", Some(110), Some(110), 5),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert_eq!(result.reason, NodeSwitchReason::BlockHeight { old_block: 100, new_block: 120 });
    }

    #[test]
    fn prioritizes_latency_on_equal_height() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 500),
            healthy_observation("https://b", Some(120), Some(120), 400),
            healthy_observation("https://c", Some(120), Some(120), 100),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.observation.url.url, "https://c");
        assert_eq!(
            result.reason,
            NodeSwitchReason::Latency {
                old_latency_ms: 500,
                new_latency_ms: 100
            }
        );
    }

    #[test]
    fn ignores_unhealthy_nodes() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            healthy_observation("https://b", Some(120), Some(120), 40),
            NodeStatusObservation::new(url("https://c"), NodeStatusState::error("rpc error"), Duration::from_millis(5)),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
    }

    #[test]
    fn reports_none_when_no_candidate() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("rpc"), Duration::from_millis(5)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).is_none());
    }

    #[test]
    fn switches_when_current_node_has_error() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            healthy_observation("https://b", Some(120), Some(120), 40),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert_eq!(
            result.reason,
            NodeSwitchReason::CurrentNodeError {
                message: "connection failed".to_string()
            }
        );
    }

    #[test]
    fn returns_none_when_current_node_not_found() {
        let current = url("https://a");
        let observations = vec![healthy_observation("https://b", Some(120), Some(120), 40)];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).is_none());
    }

    #[test]
    fn returns_none_when_current_has_error_and_no_healthy_candidates() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("also failed"), Duration::from_millis(20)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).is_none());
    }

    #[test]
    fn block_height_within_threshold_returns_latency() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 500),
            healthy_observation("https://b", Some(102), Some(102), 100),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(
            result.reason,
            NodeSwitchReason::Latency {
                old_latency_ms: 500,
                new_latency_ms: 100
            }
        );
    }

    #[test]
    fn block_height_exceeds_threshold_returns_block_height() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            healthy_observation("https://b", Some(115), Some(115), 30),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.reason, NodeSwitchReason::BlockHeight { old_block: 100, new_block: 115 });
    }

    #[test]
    fn skips_latency_switch_below_threshold() {
        let mut cfg = config();
        cfg.latency_threshold = Some(Duration::from_millis(50));
        cfg.latency_threshold_percent = Some(20.0);

        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 200),
            healthy_observation("https://b", Some(120), Some(120), 195),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &cfg, Chain::Ethereum).is_none());
    }

    #[test]
    fn allows_latency_switch_above_threshold() {
        let mut cfg = config();
        cfg.latency_threshold = Some(Duration::from_millis(50));
        cfg.latency_threshold_percent = Some(20.0);

        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 400),
            healthy_observation("https://b", Some(120), Some(120), 200),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &cfg, Chain::Ethereum).unwrap();
        assert_eq!(
            result.reason,
            NodeSwitchReason::Latency {
                old_latency_ms: 400,
                new_latency_ms: 200
            }
        );
    }

    #[test]
    fn skips_latency_switch_below_percent_threshold() {
        let mut cfg = config();
        cfg.latency_threshold = Some(Duration::from_millis(10));
        cfg.latency_threshold_percent = Some(30.0);

        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 400),
            healthy_observation("https://b", Some(120), Some(120), 350),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &cfg, Chain::Ethereum).is_none());
    }

    #[test]
    fn skips_switch_to_slower_node() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 300),
            healthy_observation("https://b", Some(120), Some(120), 700),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).is_none());
    }

    #[test]
    fn not_in_sync_switches_to_synced_even_if_slower() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 100),
            healthy_observation("https://b", Some(100), Some(100), 500),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations, &config(), Chain::Ethereum).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
    }
}
