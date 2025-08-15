use primitives::{AssetBalance, AssetId, Balance};

// UniFFI wrapper types that derive from primitives
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub balance: GemBalance,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBalance {
    pub available: String,
    pub frozen: String,
    pub locked: String,
    pub staked: String,
    pub pending: String,
    pub rewards: String,
    pub reserved: String,
    pub withdrawable: String,
}

// Conversion from primitives to UniFFI types
impl From<AssetBalance> for GemAssetBalance {
    fn from(value: AssetBalance) -> Self {
        Self {
            asset_id: value.asset_id,
            balance: value.balance.into(),
        }
    }
}

impl From<Balance> for GemBalance {
    fn from(value: Balance) -> Self {
        Self {
            available: value.available,
            frozen: value.frozen,
            locked: value.locked,
            staked: value.staked,
            pending: value.pending,
            rewards: value.rewards,
            reserved: value.reserved,
            withdrawable: value.withdrawable,
        }
    }
}

impl GemBalance {
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

    pub fn token_balance(available: String) -> Self {
        Self::coin_balance(available)
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
