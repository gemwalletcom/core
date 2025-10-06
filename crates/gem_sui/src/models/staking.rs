#[cfg(feature = "rpc")]
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
#[cfg(feature = "rpc")]
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str};

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStakeDelegation {
    pub validator_address: String,
    pub staking_pool: String,
    pub stakes: Vec<SuiStake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiSystemState {
    pub epoch: String,
    pub epoch_start_timestamp_ms: String,
    pub epoch_duration_ms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcSuiSystemState {
    pub active_validators: Vec<ValidatorInfo>,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStake {
    pub staked_sui_id: String,
    pub status: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub principal: BigUint,
    pub stake_request_epoch: String,
    pub stake_active_epoch: String,
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub estimated_reward: Option<BigUint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidators {
    pub apys: Vec<SuiValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidator {
    pub address: String,
    pub apy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub apys: Vec<ValidatorApy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorApy {
    pub address: String,
    pub apy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo {
    pub sui_address: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStake {
    pub amount: String,
    pub staker_address: String,
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventUnstake {
    pub principal_amount: String,
    pub reward_amount: String,
    pub staker_address: String,
    pub validator_address: String,
}
