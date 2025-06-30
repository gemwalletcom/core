use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandAssetResponse {
    pub params: AlgorandAsset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct AlgorandAsset {
    pub decimals: i32,
    pub name: String,
    #[serde(rename = "unit-name")]
    pub unit_name: String,
}