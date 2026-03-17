use gem_evm::address_deserializer::deserialize_ethereum_address_checksum;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Referral {
    pub referred_by: Option<ReferredBy>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub cum_vlm: f64,
    pub referrer_state: Option<ReferrerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferredBy {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerState {
    pub data: ReferrerData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerData {
    pub referral_states: Vec<ReferralUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralUser {
    #[serde(deserialize_with = "deserialize_ethereum_address_checksum")]
    pub user: String,
}
