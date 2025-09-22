use std::time::Duration;

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

pub struct NodeSyncAnalyzer;

impl NodeSyncAnalyzer {
    pub fn is_node_healthy(observation: &NodeStatusObservation) -> bool {
        observation.state.is_healthy()
    }

    pub fn select_best_node(current: &Url, observations: &[NodeStatusObservation]) -> Option<NodeStatusObservation> {
        observations
            .iter()
            .filter(|observation| observation.url != *current)
            .filter_map(|observation| match observation.state.as_status() {
                Some(status) if status.in_sync => Some((observation, status)),
                _ => None,
            })
            .max_by(|(left_observation, left_status), (right_observation, right_status)| {
                Self::compare_candidates(left_observation, left_status, right_observation, right_status)
            })
            .map(|(observation, _)| observation.clone())
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
            urls_override: None,
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

        let best = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(best.url.url, "https://b");
    }

    #[test]
    fn prioritizes_latency_on_equal_height() {
        let current = url("https://a");
        let observations = vec![
            healthy_observation("https://b", Some(120), Some(120), 40),
            healthy_observation("https://c", Some(120), Some(120), 10),
        ];

        let best = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(best.url.url, "https://c");
    }

    #[test]
    fn ignores_unhealthy_nodes() {
        let current = url("https://a");
        let mut observations = vec![healthy_observation("https://b", Some(120), Some(120), 40)];
        observations.push(NodeStatusObservation::new(
            url("https://c"),
            NodeStatusState::error("rpc error"),
            Duration::from_millis(5),
        ));

        let best = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(best.url.url, "https://b");
    }

    #[test]
    fn reports_none_when_no_candidate() {
        let current = url("https://a");
        let observations = vec![NodeStatusObservation::new(
            url("https://b"),
            NodeStatusState::error("rpc"),
            Duration::from_millis(5),
        )];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }
}
