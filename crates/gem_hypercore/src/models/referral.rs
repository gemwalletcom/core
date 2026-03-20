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
    pub referral_states: Option<Vec<ReferralUser>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralUser {
    #[serde(deserialize_with = "deserialize_ethereum_address_checksum")]
    pub user: String,
}

#[cfg(test)]
mod tests {
    use super::Referral;

    #[test]
    fn test_deserialize_referral_without_referral_states() {
        let referral: Referral = serde_json::from_str(include_str!("../../testdata/referral_need_to_trade.json")).unwrap();

        assert_eq!(referral.cum_vlm, 0.0);
        assert!(referral.referrer_state.unwrap().data.referral_states.is_none());
    }
}
