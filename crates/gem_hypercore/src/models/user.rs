use serde::{Deserialize, Serialize};

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
    pub user_cross_rate: String,
    pub active_referral_discount: String,
}
