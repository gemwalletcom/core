use primitives::AssetId;

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
