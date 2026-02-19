use std::time::Duration;

use primitives::{NodeStatusState, NodeSyncStatus};

use crate::config::Url;
use crate::monitoring::NodeStatusObservation;

pub fn url(host: &str) -> Url {
    Url {
        url: host.to_string(),
        headers: None,
    }
}

pub fn healthy_observation(host: &str, latest: Option<u64>, current: Option<u64>, latency_ms: u64) -> NodeStatusObservation {
    let status = NodeSyncStatus::new(true, latest, current);
    NodeStatusObservation::new(url(host), NodeStatusState::healthy(status), Duration::from_millis(latency_ms))
}

pub fn not_in_sync_observation(host: &str, latest: Option<u64>, current: Option<u64>, latency_ms: u64) -> NodeStatusObservation {
    let status = NodeSyncStatus::new(false, latest, current);
    NodeStatusObservation::new(url(host), NodeStatusState::healthy(status), Duration::from_millis(latency_ms))
}
