use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub max_memory_mb: usize,
    #[serde(default)]
    pub rules: HashMap<String, Vec<CacheRule>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheRule {
    pub path: Option<String>,
    pub method: Option<String>,
    pub rpc_method: Option<String>,
    pub ttl_seconds: u64,
}
