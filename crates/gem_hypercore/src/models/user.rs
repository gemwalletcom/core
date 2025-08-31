use serde::{Deserialize, Serialize};
use serde_serializers::f64::deserialize_f64_from_str;

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreUserRole {
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreAgentSession {
    pub address: String,
    pub valid_until: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreUserFee {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub user_cross_rate: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub active_referral_discount: f64,
}
