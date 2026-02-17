use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub prefix: String,
}
