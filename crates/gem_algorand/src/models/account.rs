use serde::{Deserialize, Serialize};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandAccount {
    pub amount: UInt64,
    pub assets: Vec<AlgorandAccountAsset>,
    #[serde(rename = "min-balance")]
    pub min_balance: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandAccountAsset {
    pub amount: UInt64,
    #[serde(rename = "asset-id")]
    pub asset_id: i32,
}
