use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, asset_balance::BalanceMetadata};

pub type GemAssetBalance = AssetBalance;
pub type GemBalance = Balance;
pub type GemBalanceMetadata = BalanceMetadata;

#[uniffi::remote(Record)]
pub struct GemAssetBalance {
    pub asset_id: AssetId,
    pub balance: GemBalance,
    pub is_active: bool,
}

#[uniffi::remote(Record)]
pub struct GemBalance {
    pub available: BigUint,
    pub frozen: BigUint,
    pub locked: BigUint,
    pub staked: BigUint,
    pub pending: BigUint,
    pub rewards: BigUint,
    pub reserved: BigUint,
    pub withdrawable: BigUint,
    pub metadata: Option<GemBalanceMetadata>,
}

#[uniffi::remote(Record)]
pub struct GemBalanceMetadata {
    pub energy_available: u64,
    pub energy_total: u64,
    pub bandwidth_available: u64,
    pub bandwidth_total: u64,
}
