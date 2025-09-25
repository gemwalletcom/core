use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Referral {
    pub referred_by: Option<ReferredBy>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_vlm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferredBy {
    pub code: String,
}
