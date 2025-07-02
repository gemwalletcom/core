use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use super::{Int64, UInt64};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronAccountRequest {
    pub address: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronAccount {
    pub balance: Option<UInt64>,
    pub address: Option<String>,
    pub active_permission: Option<Vec<TronAccountPermission>>,
    pub votes: Option<Vec<TronVote>>,
    #[serde(rename = "frozenV2")]
    pub frozen_v2: Option<Vec<TronFrozen>>,
    #[serde(rename = "unfrozenV2")]
    pub unfrozen_v2: Option<Vec<TronUnfrozen>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronAccountPermission {
    pub threshold: Int64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TronAccountUsage {
    pub free_net_used: Option<Int64>,
    pub free_net_limit: Option<Int64>,
    #[serde(rename = "EnergyUsed")]
    pub energy_used: Option<Int64>,
    #[serde(rename = "EnergyLimit")]
    pub energy_limit: Option<Int64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronEmptyAccount {
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronVote {
    pub vote_address: String,
    pub vote_count: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronFrozen {
    #[serde(rename = "type")]
    pub frozen_type: Option<String>,
    pub amount: Option<UInt64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronUnfrozen {
    pub unfreeze_amount: Option<UInt64>,
    pub unfreeze_expire_time: Option<UInt64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TronReward {
    pub reward: Option<UInt64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct WitnessesList {
    pub witnesses: Vec<WitnessAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WitnessAccount {
    pub address: String,
    pub vote_count: Option<UInt64>,
    pub url: String,
    pub is_jobs: Option<bool>,
}
