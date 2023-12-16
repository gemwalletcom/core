use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "camelCase")]
pub struct FiatAssets {
    pub version: u32,
    pub asset_ids: Vec<String>,
}
