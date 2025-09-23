use primitives::{asset_balance::BalanceMetadata, AssetBalance, AssetId, Balance};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub balance: GemBalance,
    pub is_active: bool,
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
    pub metadata: Option<GemBalanceMetadata>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBalanceMetadata {
    pub energy_available: u64,
    pub energy_total: u64,
    pub bandwidth_available: u64,
    pub bandwidth_total: u64,
}

impl From<AssetBalance> for GemAssetBalance {
    fn from(value: AssetBalance) -> Self {
        Self {
            asset_id: value.asset_id,
            balance: value.balance.into(),
            is_active: value.is_active,
        }
    }
}

impl From<Balance> for GemBalance {
    fn from(value: Balance) -> Self {
        Self {
            available: value.available.to_string(),
            frozen: value.frozen.to_string(),
            locked: value.locked.to_string(),
            staked: value.staked.to_string(),
            pending: value.pending.to_string(),
            rewards: value.rewards.to_string(),
            reserved: value.reserved.to_string(),
            withdrawable: value.withdrawable.to_string(),
            metadata: value.metadata.map(|m| m.into()),
        }
    }
}

impl From<BalanceMetadata> for GemBalanceMetadata {
    fn from(value: BalanceMetadata) -> Self {
        Self {
            energy_available: value.energy_available,
            energy_total: value.energy_total,
            bandwidth_available: value.bandwidth_available,
            bandwidth_total: value.bandwidth_total,
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
            metadata: None,
        }
    }
}