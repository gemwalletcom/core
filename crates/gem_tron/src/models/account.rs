use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronAccountRequest {
    pub address: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronAccount {
    pub balance: Option<u64>,
    pub address: Option<String>,
    pub active_permission: Option<Vec<TronAccountPermission>>,
    pub votes: Option<Vec<TronVote>>,
    #[serde(rename = "frozenV2")]
    pub frozen_v2: Option<Vec<TronFrozen>>,
    #[serde(rename = "unfrozenV2")]
    pub unfrozen_v2: Option<Vec<TronUnfrozen>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronAccountPermission {
    pub threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronAccountUsage {
    pub free_net_used: Option<u64>,
    pub free_net_limit: Option<u64>,
    #[serde(rename = "EnergyUsed")]
    pub energy_used: Option<u64>,
    #[serde(rename = "EnergyLimit")]
    pub energy_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronEmptyAccount {
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronVote {
    pub vote_address: String,
    pub vote_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronFrozen {
    #[serde(rename = "type")]
    pub frozen_type: Option<String>,
    pub amount: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronUnfrozen {
    pub unfreeze_amount: Option<u64>,
    pub unfreeze_expire_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronReward {
    pub reward: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessesList {
    pub witnesses: Vec<WitnessAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WitnessAccount {
    pub address: String,
    pub vote_count: Option<u64>,
    pub url: String,
    pub is_jobs: Option<bool>,
}
