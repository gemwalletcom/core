use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyLimits {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub max: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub min: f64,
}