use std::time::Duration;

use crate::config::{AdaptiveMonitoringConfig, ErrorMatcherConfig, NodeMonitoringConfig, RetryConfig};

pub fn monitoring_config() -> NodeMonitoringConfig {
    NodeMonitoringConfig {
        enabled: true,
        poll_interval_seconds: Duration::from_secs(600),
        block_delay: 100,
        adaptive: adaptive_monitoring_config(),
    }
}

pub fn adaptive_monitoring_config() -> AdaptiveMonitoringConfig {
    AdaptiveMonitoringConfig {
        enabled: true,
        window: Duration::from_secs(30),
        min_samples: 20,
        error_threshold: 0.5,
        recovery_threshold: 0.2,
        cooldown: Duration::from_secs(45),
        min_switch_interval: Duration::from_secs(15),
        errors: error_matcher_config(vec![429], vec!["rate limit"]),
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
