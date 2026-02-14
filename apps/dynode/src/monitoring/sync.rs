use std::time::Duration;

use super::switch_reason::NodeSwitchReason;
use crate::config::Url;
use primitives::{NodeStatusState, NodeSyncStatus};

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
        observation.state.is_healthy()
    }

    pub fn select_best_node(current: &Url, observations: &[NodeStatusObservation]) -> Option<NodeSwitchResult> {
        let current_observation = observations.iter().find(|o| o.url == *current)?;

        let switch_reason = match &current_observation.state {
            NodeStatusState::Error { message } => Some(NodeSwitchReason::CurrentNodeError { message: message.clone() }),
            NodeStatusState::Healthy(_) => None,
        };

        let best_candidate = observations
            .iter()
            .filter(|observation| observation.url != *current)
            .filter_map(|observation| match observation.state.as_status() {
                Some(status) if status.in_sync => Some((observation, status)),
                _ => None,
            })
            .max_by(|(left_observation, left_status), (right_observation, right_status)| Self::compare_candidates(left_observation, left_status, right_observation, right_status));

        match (switch_reason, best_candidate) {
            (Some(reason), Some((observation, _))) => Some(NodeSwitchResult {
                observation: observation.clone(),
                reason,
            }),
            (None, Some((observation, new_status))) => {
                let current_status = current_observation.state.as_status()?;
                Some(NodeSwitchResult {
                    observation: observation.clone(),
                    reason: Self::switch_reason(current_status, new_status, current_observation.latency, observation.latency),
                })
            }
            _ => None,
        }
    }

    fn switch_reason(current: &NodeSyncStatus, new: &NodeSyncStatus, current_latency: Duration, new_latency: Duration) -> NodeSwitchReason {
        let old_block = Self::status_height(current);
        let new_block = Self::status_height(new);
        if new_block > old_block {
            NodeSwitchReason::BlockHeight { old_block, new_block }
        } else {
            NodeSwitchReason::Latency {
                old_latency_ms: current_latency.as_millis() as u64,
                new_latency_ms: new_latency.as_millis() as u64,
            }
        }
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

    fn url(host: &str) -> Url {
        Url {
            url: host.to_string(),
            headers: None,
        }
    }

    fn healthy_observation(host: &str, latest: Option<u64>, current: Option<u64>, latency_ms: u64) -> NodeStatusObservation {
        let status = NodeSyncStatus::new(true, latest, current);
        NodeStatusObservation::new(url(host), NodeStatusState::healthy(status), Duration::from_millis(latency_ms))
    }

    #[test]
    fn selects_highest_block_number() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            healthy_observation("https://b", Some(120), Some(120), 30),
            healthy_observation("https://c", Some(110), Some(110), 5),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert!(matches!(result.reason, NodeSwitchReason::BlockHeight { old_block: 100, new_block: 120 }));
    }

    #[test]
    fn prioritizes_latency_on_equal_height() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(120), Some(120), 50),
            healthy_observation("https://b", Some(120), Some(120), 40),
            healthy_observation("https://c", Some(120), Some(120), 10),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://c");
        assert!(matches!(
            result.reason,
            NodeSwitchReason::Latency {
                old_latency_ms: 50,
                new_latency_ms: 10
            }
        ));
    }

    #[test]
    fn ignores_unhealthy_nodes() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            healthy_observation("https://b", Some(120), Some(120), 40),
            NodeStatusObservation::new(url("https://c"), NodeStatusState::error("rpc error"), Duration::from_millis(5)),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
    }

    #[test]
    fn reports_none_when_no_candidate() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://a", Some(100), Some(100), 10),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("rpc"), Duration::from_millis(5)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }

    #[test]
    fn switches_when_current_node_has_error() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            healthy_observation("https://b", Some(120), Some(120), 40),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert!(matches!(result.reason, NodeSwitchReason::CurrentNodeError { .. }));
    }

    #[test]
    fn returns_none_when_current_node_not_found() {
        let current = url("https://a");
        let observations = vec![healthy_observation("https://b", Some(120), Some(120), 40)];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }

    #[test]
    fn returns_none_when_current_has_error_and_no_healthy_candidates() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("also failed"), Duration::from_millis(20)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }
}
