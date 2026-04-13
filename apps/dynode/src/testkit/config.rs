use std::time::Duration;

use crate::config::{ErrorMatcherConfig, NodeMonitoringConfig, RetryConfig};

pub fn monitoring_config() -> NodeMonitoringConfig {
    NodeMonitoringConfig {
        enabled: true,
        poll_interval: Duration::from_secs(600),
        max_sync_delay: Duration::from_secs(24),
        max_sync_blocks: 20,
        latency_threshold: Some(Duration::from_millis(250)),
        latency_threshold_percent: Some(20.0),
    }
}

pub fn retry_config(enabled: bool, status_codes: Vec<u16>, error_messages: Vec<&str>) -> RetryConfig {
    retry_config_with_attempts(enabled, 0, status_codes, error_messages)
}

pub fn retry_config_with_attempts(enabled: bool, max_attempts: usize, status_codes: Vec<u16>, error_messages: Vec<&str>) -> RetryConfig {
    RetryConfig {
        enabled,
        max_attempts,
        errors: error_matcher_config(status_codes, error_messages),
    }
}

pub fn error_matcher_config(status_codes: Vec<u16>, error_messages: Vec<&str>) -> ErrorMatcherConfig {
    ErrorMatcherConfig {
        status_codes,
        error_messages: error_messages.into_iter().map(|value| value.to_string()).collect(),
    }
}
