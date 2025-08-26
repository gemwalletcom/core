use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub amount: u64,
    pub assets: Vec<AccountAsset>,
    #[serde(rename = "min-balance")]
    pub min_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAsset {
    pub amount: u64,
    #[serde(rename = "asset-id")]
    pub asset_id: i32,
}
