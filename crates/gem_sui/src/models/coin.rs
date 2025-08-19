use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoin {
    pub coin_type: String,
    pub coin_object_id: String,
    pub balance: String,
    pub version: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinBalance {
    pub coin_type: String,
    pub total_balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinMetadata {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}
