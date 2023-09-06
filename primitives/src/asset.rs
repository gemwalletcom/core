use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::{asset_id::AssetId, asset_type::AssetType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Asset {
    pub id: AssetId,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "type")]
    pub asset_type: AssetType
}