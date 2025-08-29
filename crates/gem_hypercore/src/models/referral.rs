use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreReferral {
    pub referred_by: Option<HypercoreReferredBy>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_vlm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreReferredBy {
    pub code: String,
}
