use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct SettingsOperator {
    pub appstore: OperatorAppStore,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct OperatorAppStore {
    pub refresh_interval_ms: u64,
    pub timeout_ms: u64,
    pub keys: Vec<String>,
    pub apps: Vec<OperatorAppStoreApp>,
    pub languages: Vec<OperatorAppStoreLanguage>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct OperatorAppStoreApp {
    pub name: String,
    pub id: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct OperatorAppStoreLanguage {
    pub name: String,
    pub code: String,
}

impl SettingsOperator {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().unwrap();
        let setting_path = current_dir.join("SettingsOperator.yaml");
        let s = Config::builder()
            .add_source(File::from(setting_path))
            .add_source(Environment::with_prefix("").prefix_separator("").separator("_"))
            .build()?;
        s.try_deserialize()
    }
}
