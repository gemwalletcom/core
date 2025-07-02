use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::account::CosmosBalance;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosDelegations {
    pub delegation_responses: Vec<CosmosDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosDelegation {
    pub delegation: CosmosDelegationData,
    pub balance: CosmosBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosDelegationData {
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosUnboundingDelegations {
    pub unbonding_responses: Vec<CosmosUnboundingDelegation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosUnboundingDelegation {
    pub validator_address: String,
    pub entries: Vec<CosmosUnboudingDelegationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosUnboudingDelegationEntry {
    pub completion_time: String,
    pub creation_height: String,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosRewards {
    pub rewards: Vec<CosmosReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosReward {
    pub validator_address: String,
    pub reward: Vec<CosmosBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosValidators {
    pub validators: Vec<CosmosValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosValidator {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: CosmosValidatorMoniker,
    pub commission: CosmosValidatorCommission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosValidatorMoniker {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosValidatorCommission {
    pub commission_rates: CosmosValidatorCommissionRates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct CosmosValidatorCommissionRates {
    pub rate: String,
}
