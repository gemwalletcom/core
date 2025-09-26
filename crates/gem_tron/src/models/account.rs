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
    pub owner_permission: Option<TronAccountOwnerPermission>,
    pub active_permission: Option<Vec<TronAccountPermission>>,
    pub votes: Option<Vec<TronVote>>,
    #[serde(rename = "frozenV2")]
    pub frozen_v2: Option<Vec<TronFrozen>>,
    #[serde(rename = "unfrozenV2")]
    pub unfrozen_v2: Option<Vec<TronUnfrozen>>,
}

impl TronAccount {
    pub fn is_staking(&self) -> bool {
        self.frozen_v2.as_ref().unwrap_or(&vec![]).iter().filter(|x| x.amount > 0).count() > 0
            || self.unfrozen_v2.as_ref().unwrap_or(&vec![]).iter().filter(|x| x.unfreeze_amount > 0).count() > 0
            || self.votes.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronAccountPermission {
    pub threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronAccountOwnerPermission {
    pub permission_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TronAccountUsage {
    #[serde(default)]
    pub free_net_used: u64,
    #[serde(default)]
    pub free_net_limit: u64,
    #[serde(rename = "NetUsed", default)]
    pub net_used: u64,
    #[serde(rename = "NetLimit", default)]
    pub net_limit: u64,
    #[serde(rename = "EnergyUsed", default)]
    pub energy_used: u64,
    #[serde(rename = "EnergyLimit", default)]
    pub energy_limit: u64,
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
    #[serde(default)]
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronUnfrozen {
    #[serde(default)]
    pub unfreeze_amount: u64,
    pub unfreeze_expire_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronReward {
    pub reward: u64,
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
