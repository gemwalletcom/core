use std::{collections::HashMap, env};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::node_service::NodeResult;
use primitives::Chain;

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: Metrics,
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl NodeConfig {
    pub fn domains_map(&self) -> HashMap<String, Domain> {
        let mut map: HashMap<String, Domain> = HashMap::new();
        for domain in &self.domains {
            map.insert(domain.domain.clone(), domain.clone());
        }
        map
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Metrics {
    pub port: u16,
    pub address: String,
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub user_agent_patterns: UserAgentPatterns,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UserAgentPatterns {
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CacheConfig {
    pub max_memory_mb: usize,
    #[serde(default)]
    pub rules: HashMap<String, Vec<CacheRule>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheRule {
    pub path: Option<String>,
    pub method: Option<String>,
    pub rpc_method: Option<String>,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Domain {
    pub domain: String,
    pub chain: Chain,
    pub block_delay: Option<u64>,
    pub poll_interval_seconds: Option<u64>,
    pub urls: Vec<Url>,
}

impl Domain {
    pub fn get_poll_interval_seconds(&self) -> u64 {
        self.poll_interval_seconds.unwrap_or(600) // 10 minutes
    }

    pub fn get_block_delay(&self) -> u64 {
        self.block_delay.unwrap_or(100)
    }

    pub fn is_url_behind(&self, url: Url, results: Vec<NodeResult>) -> bool {
        if let Some(index) = results.iter().position(|r| r.url == url) {
            let node = results[index].clone();
            if let Some(max_block_number) = Self::find_highest_block_number(results) {
                if node.block_number + self.get_block_delay() >= max_block_number.block_number {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_highest_block_number(results: Vec<NodeResult>) -> Option<NodeResult> {
        results.into_iter().max_by(|x, y| x.block_number.cmp(&y.block_number))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Url {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub urls_override: Option<HashMap<String, Url>>,
}

impl NodeConfig {
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
