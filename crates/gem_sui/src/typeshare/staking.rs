use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStakeDelegation {
    pub validator_address: String,
    pub staking_pool: String,
    pub stakes: Vec<SuiStake>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiSystemState {
    pub epoch: String,
    pub epoch_start_timestamp_ms: String,
    pub epoch_duration_ms: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStake {
    pub staked_sui_id: String,
    pub status: String,
    pub principal: String,
    pub stake_request_epoch: String,
    pub stake_active_epoch: String,
    pub estimated_reward: Option<String>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidators {
    pub apys: Vec<SuiValidator>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidator {
    pub address: String,
    pub apy: f64,
}
