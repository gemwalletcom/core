use crate::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: String,
}
