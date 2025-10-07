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
    pub pending: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub rewards_rate: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub rewards_rate_denominator: u64,
}
