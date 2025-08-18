use serde::{Deserialize, Serialize};

use super::account::CosmosBalance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDelegations {
    pub delegation_responses: Vec<CosmosDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDelegation {
    pub delegation: CosmosDelegationData,
    pub balance: CosmosBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDelegationData {
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosUnboundingDelegations {
    pub unbonding_responses: Vec<CosmosUnboundingDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosUnboundingDelegation {
    pub validator_address: String,
    pub entries: Vec<CosmosUnboudingDelegationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosUnboudingDelegationEntry {
    pub completion_time: String,
    pub creation_height: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosRewards {
    pub rewards: Vec<CosmosReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosReward {
    pub validator_address: String,
    pub reward: Vec<CosmosBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosValidators {
    pub validators: Vec<CosmosValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosValidator {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: CosmosValidatorMoniker,
    pub commission: CosmosValidatorCommission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosValidatorMoniker {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosValidatorCommission {
    pub commission_rates: CosmosValidatorCommissionRates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosValidatorCommissionRates {
    pub rate: String,
}
