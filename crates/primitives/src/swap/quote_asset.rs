use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Chain};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare]
pub struct QuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}

impl QuoteAsset {
    pub fn asset_id(&self) -> AssetId {
        AssetId::new(&self.id).unwrap()
    }

    pub fn is_native(&self) -> bool {
        self.asset_id().is_native()
    }

    pub fn chain(&self) -> Chain {
        self.asset_id().chain
    }
}

impl From<AssetId> for QuoteAsset {
    fn from(id: AssetId) -> Self {
        Self {
            id: id.to_string(),
            symbol: String::new(),
            decimals: 0,
        }
    }
}
