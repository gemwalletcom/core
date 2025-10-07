use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_u64_from_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub active_validators: Vec<ValidatorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub addr: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub voting_power: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPoolStake {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub active: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub inactive: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub pending_active: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub pending_inactive: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub rewards_rate: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub rewards_rate_denominator: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub recurring_lockup_duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconfigurationState {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub epoch: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub last_reconfiguration_time: u64,
}
