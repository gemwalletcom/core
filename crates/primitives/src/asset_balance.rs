use crate::AssetId;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: Balance,
    pub is_active: Option<bool>,
}

impl AssetBalance {
    pub fn new(asset_id: AssetId, balance: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::coin_balance(balance),
            is_active: None,
        }
    }

    pub fn new_balance(asset_id: AssetId, balance: Balance) -> Self {
        Self {
            asset_id,
            balance,
            is_active: None,
        }
    }

    pub fn new_with_active(asset_id: AssetId, balance: Balance, is_active: bool) -> Self {
        Self {
            asset_id,
            balance,
            is_active: Some(is_active),
        }
    }

    pub fn new_staking(asset_id: AssetId, staked: BigUint, pending: BigUint, rewards: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::stake_balance(staked, pending, Some(rewards)),
            is_active: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub available: BigUint,
    pub frozen: BigUint,
    pub locked: BigUint,
    pub staked: BigUint,
    pub pending: BigUint,
    pub rewards: BigUint,
    pub reserved: BigUint,
    pub withdrawable: BigUint,
}

impl Balance {
    pub fn coin_balance(available: BigUint) -> Self {
        Self {
            available,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            reserved: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
        }
    }

    pub fn with_reserved(available: BigUint, reserved: BigUint) -> Self {
        Self {
            available,
            reserved,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
        }
    }

    pub fn stake_balance(staked: BigUint, pending: BigUint, rewards: Option<BigUint>) -> Self {
        Self {
            available: BigUint::from(0u32),
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked,
            pending,
            rewards: rewards.unwrap_or(BigUint::from(0u32)),
            reserved: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
        }
    }
}
