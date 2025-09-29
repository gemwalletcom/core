use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MetricsConfig {
    #[serde(default)]
    pub user_agent_patterns: UserAgentPatterns,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserAgentPatterns {
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,
}
