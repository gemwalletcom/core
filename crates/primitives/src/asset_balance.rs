use crate::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: String,
}

impl AssetBalance {
    pub fn new(asset_id: AssetId, balance: String) -> Self {
        Self { asset_id, balance }
    }
}
