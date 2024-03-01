use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatAssets {
    pub version: u32,
    pub asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAsset {
    pub asset_id: AssetId,
    pub provider: String,
    pub symbol: String,
    pub network: Option<String>,
    pub enabled: bool,
}
