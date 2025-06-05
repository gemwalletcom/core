use serde::{Deserialize, Serialize};

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetAddress {
    pub asset_id: AssetId,
    pub address: String,
}

impl AssetAddress {
    pub fn new(asset_id: AssetId, address: String) -> Self {
        Self { asset_id, address }
    }
}
