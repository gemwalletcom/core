use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "asset-id")]
    pub asset_id: i64,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub asset: AssetDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDetails {
    pub index: i64,
    pub params: AssetParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetParams {
    pub decimals: i64,
    pub name: String,
    #[serde(rename = "unit-name")]
    pub unit_name: String,
}