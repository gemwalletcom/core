use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError, Environment, File};
use primitives::Chain;
use serde::Deserialize;

mod cache;
mod domain;
mod metrics;
mod url;

pub use cache::{CacheConfig, CacheRule};
pub use domain::ChainConfig;
pub use metrics::{MetricsConfig, UserAgentPatterns};
pub use url::{NodeResult, Override, Url};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeMonitoringConfig {
    pub enabled: bool,
    pub poll_interval_seconds: u64,
    pub block_delay: u64,
}

impl Default for NodeMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
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
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsSettings,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub monitoring: NodeMonitoringConfig,
    pub retry: RetryConfig,
    pub request: RequestConfig,
}

#[derive(Debug, Deserialize)]
struct ChainsFile {
    chains: Vec<ChainConfig>,
}

pub fn load_config() -> Result<(NodeConfig, HashMap<Chain, ChainConfig>), ConfigError> {
    let current_dir = env::current_dir().unwrap();

    let base_dir = if current_dir.join("config.yml").exists() {
        current_dir
    } else {
        current_dir.join("apps/dynode")
    };

    let config: NodeConfig = Config::builder()
        .add_source(File::from(base_dir.join("config.yml")))
        .add_source(Environment::default().separator("_"))
        .build()?
        .try_deserialize()?;

    let chains = find_chain_files(&base_dir)
        .into_iter()
        .map(|path| Config::builder().add_source(File::from(path)).build()?.try_deserialize::<ChainsFile>())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flat_map(|cf| cf.chains)
        .map(|c| (c.chain, c))
        .collect();

    Ok((config, chains))
}

fn find_chain_files(base_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(base_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("chains") && name.ends_with(".yml"))
        })
        .collect();

    files.sort();
    files
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsSettings {
    pub port: u16,
    pub address: String,
    pub prefix: String,
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}
