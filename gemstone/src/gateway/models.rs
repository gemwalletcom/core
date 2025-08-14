#[derive(Debug, Clone, uniffi::Record)]
pub struct AssetBalanceWrapper {
    pub asset_id: String,
    pub balance: BalanceWrapper,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct BalanceWrapper {
    pub available: String,
    pub frozen: String,
    pub locked: String,
    pub staked: String,
    pub pending: String,
    pub rewards: String,
    pub reserved: String,
    pub withdrawable: String,
}
