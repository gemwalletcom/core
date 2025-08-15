use crate::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: Balance,
}

impl AssetBalance {
    pub fn new(asset_id: AssetId, balance: String) -> Self {
        Self {
            asset_id,
            balance: Balance::coin_balance(balance),
        }
    }

    pub fn new_balance(asset_id: AssetId, balance: Balance) -> Self {
        Self { asset_id, balance }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub available: String,
    pub frozen: String,
    pub locked: String,
    pub staked: String,
    pub pending: String,
    pub rewards: String,
    pub reserved: String,
    pub withdrawable: String,
}

impl Balance {
    pub fn coin_balance(available: String) -> Self {
        Self {
            available,
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked: "0".to_string(),
            pending: "0".to_string(),
            rewards: "0".to_string(),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }

    pub fn stake_balance(staked: String, pending: String, rewards: Option<String>) -> Self {
        Self {
            available: "0".to_string(),
            frozen: "0".to_string(),
            locked: "0".to_string(),
            staked,
            pending,
            rewards: rewards.unwrap_or("0".to_string()),
            reserved: "0".to_string(),
            withdrawable: "0".to_string(),
        }
    }
}
