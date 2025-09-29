use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub prefix: String,
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            user_agent_patterns: UserAgentPatterns::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserAgentPatterns {
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,
}
