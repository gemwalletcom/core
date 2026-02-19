use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use config::{Config, ConfigError, Environment, File};
use primitives::Chain;
use serde::Deserialize;
use serde_serializers::duration;

mod cache;
mod domain;
mod metrics;
mod url;

pub use cache::{CacheConfig, CacheRule};
pub use domain::ChainConfig;
pub use metrics::MetricsConfig;
pub use url::{NodeResult, Override, Url};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeMonitoringConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "duration::deserialize")]
    pub poll_interval: Duration,
    #[serde(deserialize_with = "duration::deserialize")]
    pub max_sync_delay: Duration,
    pub max_sync_blocks: u64,
    #[serde(default, deserialize_with = "duration::deserialize_option")]
    pub latency_threshold: Option<Duration>,
    #[serde(default)]
    pub latency_threshold_percent: Option<f64>,
    pub adaptive: AdaptiveMonitoringConfig,
}

impl NodeMonitoringConfig {
    pub fn get_poll_interval_seconds(&self) -> u64 {
        self.poll_interval.as_secs()
    }

    pub fn block_delay_threshold(&self, chain: Chain) -> u64 {
        let block_time_ms = chain.block_time() as u64;
        if block_time_ms == 0 {
            return 1;
        }
        let computed = self.max_sync_delay.as_millis() as u64 / block_time_ms;
        computed.clamp(1, self.max_sync_blocks)
    }

    pub fn is_latency_improvement_significant(&self, old: Duration, new: Duration) -> bool {
        if new >= old {
            return false;
        }
        let diff = old - new;
        if let Some(threshold) = self.latency_threshold
            && diff < threshold
        {
            return false;
        }
        if let Some(percent) = self.latency_threshold_percent
            && (diff.as_millis() as f64 / old.as_millis() as f64) * 100.0 < percent
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdaptiveMonitoringConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "duration::deserialize")]
    pub window: Duration,
    pub min_samples: usize,
    pub error_threshold: f64,
    pub recovery_threshold: f64,
    #[serde(deserialize_with = "duration::deserialize")]
    pub cooldown: Duration,
    #[serde(deserialize_with = "duration::deserialize")]
    pub min_switch_interval: Duration,
    pub errors: ErrorMatcherConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ErrorMatcherConfig {
    pub status_codes: Vec<u16>,
    pub error_messages: Vec<String>,
}

impl ErrorMatcherConfig {
    pub fn matches_status(&self, status: u16) -> bool {
        self.status_codes.contains(&status)
    }

    pub fn matches_message(&self, message: &str) -> bool {
        if message.is_empty() {
            return false;
        }

        let message_lower = message.to_ascii_lowercase();
        self.error_messages.iter().any(|pattern| {
            let pattern = pattern.trim();
            !pattern.is_empty() && message_lower.contains(&pattern.to_ascii_lowercase())
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    pub enabled: bool,
    pub max_attempts: usize,
    pub errors: ErrorMatcherConfig,
}

impl RetryConfig {
    pub fn matches_status(&self, status: u16) -> bool {
        self.errors.matches_status(status)
    }

    pub fn matches_message(&self, message: &str) -> bool {
        self.errors.matches_message(message)
    }

    pub fn effective_max_attempts(&self, urls_count: usize) -> usize {
        if self.max_attempts == 0 { urls_count } else { self.max_attempts }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_matcher;
    use crate::testkit::config as testkit;

    #[test]
    fn test_should_retry_on_error_message() {
        let config = testkit::retry_config(true, vec![], vec!["daily request limit", "rate limit"]);

        assert!(config.matches_message("daily request limit reached - upgrade your account"));
        assert!(config.matches_message("rate limit exceeded"));
        assert!(config.matches_message("Rate Limit Exceeded"));
        assert!(!config.matches_message("internal server error"));
        assert!(!config.matches_message(""));
    }

    #[test]
    fn test_should_retry_on_error_message_empty() {
        let config = testkit::retry_config(true, vec![], vec![]);

        assert!(!config.matches_message("daily request limit reached"));
    }

    #[test]
    fn test_matches_status() {
        let config = testkit::retry_config(true, vec![401, 403, 429], vec![]);
        assert!(config.matches_status(429));
        assert!(!config.matches_status(500));
    }

    #[test]
    fn test_matches_message_case_insensitive_without_normalize() {
        let config = testkit::retry_config(true, vec![], vec!["RATE LIMIT"]);
        assert!(config.matches_message("rate limit exceeded"));
    }

    #[test]
    fn test_normalize_patterns() {
        let mut config = testkit::retry_config(true, vec![], vec![" Rate Limit ", "", "rate limit"]);
        normalize_matcher(&mut config.errors);
        assert_eq!(config.errors.error_messages, vec!["rate limit".to_string()]);
        assert!(config.matches_message("RATE LIMIT EXCEEDED"));
    }

    #[test]
    fn test_effective_max_attempts() {
        let config_zero = testkit::retry_config(true, vec![], vec![]);
        assert_eq!(config_zero.effective_max_attempts(5), 5);
        assert_eq!(config_zero.effective_max_attempts(10), 10);

        let config_limited = testkit::retry_config_with_attempts(true, 3, vec![], vec![]);
        assert_eq!(config_limited.effective_max_attempts(5), 3);
        assert_eq!(config_limited.effective_max_attempts(2), 3);
    }

    #[test]
    fn test_block_delay_threshold() {
        use primitives::Chain;

        let config = testkit::monitoring_config();

        assert_eq!(config.block_delay_threshold(Chain::Ethereum), 2);
        assert_eq!(config.block_delay_threshold(Chain::Bitcoin), 1);
        assert_eq!(config.block_delay_threshold(Chain::Solana), 20);
        assert_eq!(config.block_delay_threshold(Chain::SmartChain), 20);
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequestConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    pub timeout: Duration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeadersConfig {
    pub forward: Vec<String>,
    #[serde(default)]
    pub domains: HashMap<String, Vec<String>>,
}

impl HeadersConfig {
    pub fn get_domain_headers(&self, host: &str) -> Option<&Vec<String>> {
        self.domains.get(host)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsSettings,
    #[serde(default)]
    pub cache: CacheConfig,
    pub monitoring: NodeMonitoringConfig,
    pub retry: RetryConfig,
    pub request: RequestConfig,
    pub headers: HeadersConfig,
    pub jwt: JwtConfig,
}

impl NodeConfig {
    fn normalize(&mut self) {
        normalize_matcher(&mut self.retry.errors);
        normalize_matcher(&mut self.monitoring.adaptive.errors);
    }
}

fn normalize_matcher(matcher: &mut ErrorMatcherConfig) {
    normalize_error_messages(&mut matcher.error_messages);
}

fn normalize_error_messages(messages: &mut Vec<String>) {
    let mut seen = HashSet::with_capacity(messages.len());
    let mut normalized = Vec::with_capacity(messages.len());

    for message in messages.drain(..) {
        let value = message.trim().to_ascii_lowercase();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.clone()) {
            normalized.push(value);
        }
    }

    *messages = normalized;
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

    let mut config: NodeConfig = Config::builder()
        .add_source(File::from(base_dir.join("config.yml")))
        .add_source(Environment::default().separator("_"))
        .build()?
        .try_deserialize()?;
    config.normalize();

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
}
