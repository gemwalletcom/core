use std::{collections::HashMap, env};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

mod cache;
mod domain;
mod metrics;
mod url;

pub use cache::{CacheConfig, CacheRule};
pub use domain::Domain;
pub use metrics::{MetricsConfig, UserAgentPatterns};
pub use url::{NodeResult, Override, Url};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeMonitoringConfig {
    pub poll_interval_seconds: u64,
    pub block_delay: u64,
}

impl Default for NodeMonitoringConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 60 * 15,
            block_delay: 100,
        }
    }
}

impl NodeMonitoringConfig {
    pub fn get_poll_interval_seconds(&self) -> u64 {
        self.poll_interval_seconds
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    pub enabled: bool,
    pub status_codes: Vec<u16>,
    pub error_messages: Vec<String>,
}

impl RetryConfig {
    pub fn should_retry_on_error_message(&self, message: &str) -> bool {
        self.error_messages.iter().any(|prefix| message.contains(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry_on_error_message() {
        let config = RetryConfig {
            enabled: true,
            status_codes: vec![],
            error_messages: vec!["daily request limit".to_string(), "rate limit".to_string()],
        };

        assert!(config.should_retry_on_error_message("daily request limit reached - upgrade your account"));
        assert!(config.should_retry_on_error_message("rate limit exceeded"));
        assert!(!config.should_retry_on_error_message("internal server error"));
        assert!(!config.should_retry_on_error_message(""));
    }

    #[test]
    fn test_should_retry_on_error_message_empty() {
        let config = RetryConfig {
            enabled: true,
            status_codes: vec![],
            error_messages: vec![],
        };

        assert!(!config.should_retry_on_error_message("daily request limit reached"));
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequestConfig {
    pub timeout_seconds: u64,
    pub connect_timeout_seconds: u64,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 60,
            connect_timeout_seconds: 15,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsSettings,
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub monitoring: NodeMonitoringConfig,
    pub retry: RetryConfig,
    #[serde(default)]
    pub request: RequestConfig,
}

impl NodeConfig {
    pub fn domains_map(&self) -> HashMap<String, Domain> {
        let mut map: HashMap<String, Domain> = HashMap::new();
        for domain in &self.domains {
            map.insert(domain.domain.clone(), domain.clone());
        }
        map
    }

    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();

        let default_config_path = current_dir.join("config.yml");
        let config_path = if default_config_path.exists() {
            default_config_path
        } else {
            current_dir.join("apps/dynode/config.yml")
        };

        let default_domains_path = current_dir.join("domains.yml");
        let domains_path = if default_domains_path.exists() {
            default_domains_path
        } else {
            current_dir.join("apps/dynode/domains.yml")
        };

        let s = Config::builder()
            .add_source(File::from(config_path))
            .add_source(File::from(domains_path))
            .add_source(Environment::with_prefix("").prefix_separator("").separator("_"))
            .build()?;
        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsSettings {
    pub port: u16,
    pub address: String,
    pub prefix: String,
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}
