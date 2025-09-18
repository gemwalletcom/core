use std::{collections::HashMap, env};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

mod cache;
mod metrics;
mod node;

pub use cache::{CacheConfig, CacheRule};
pub use metrics::{MetricsConfig, UserAgentPatterns};
pub use node::{Domain, NodeResult, Url};

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub address: String,
    pub metrics: MetricsSettings,
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
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}
