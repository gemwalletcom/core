use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandAssetResponse {
    pub params: AlgorandAsset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorandAsset {
    pub decimals: i32,
    pub name: String,
    #[serde(rename = "unit-name")]
    pub unit_name: String,
}
