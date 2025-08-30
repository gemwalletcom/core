use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

use super::account::Balance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegations {
    pub delegation_responses: Vec<Delegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegation: DelegationData,
    pub balance: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationData {
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegations {
    pub unbonding_responses: Vec<UnbondingDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegation {
    pub validator_address: String,
    pub entries: Vec<UnbondingDelegationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegationEntry {
    pub completion_time: String,
    pub creation_height: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewards {
    pub rewards: Vec<Reward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub validator_address: String,
    pub reward: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validators {
    pub validators: Vec<ValidatorLegacy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorLegacy {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: ValidatorMoniker,
    pub commission: ValidatorCommissionLegacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMoniker {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommissionLegacy {
    pub commission_rates: ValidatorCommissionRatesLegacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommissionRatesLegacy {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: ValidatorDescription,
    pub commission: ValidatorCommission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDescription {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommission {
    pub commission_rates: ValidatorCommissionRates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommissionRates {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPoolResponse {
    pub pool: StakingPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub bonded_tokens: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub not_bonded_tokens: f64,
}
