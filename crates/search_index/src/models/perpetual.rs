use primitives::{Asset, Perpetual};
use serde::{Deserialize, Serialize};

use crate::sanitize_index_primary_id;

pub const PERPETUALS_INDEX_NAME: &str = "perpetuals";
pub const PERPETUALS_FILTERS: &[&str] = &[
    "perpetual.name",
    "perpetual.identifier",
    "perpetual.provider",
    "perpetual.price",
    "perpetual.volume24h",
];
pub const PERPETUALS_SEARCH_ATTRIBUTES: &[&str] = &["perpetual.name", "perpetual.identifier", "perpetual.provider"];
pub const PERPETUALS_RANKING_RULES: &[&str] = &["words", "typo", "perpetual.volume24h:desc", "proximity", "attribute", "exactness"];

pub const PERPETUALS_SORTS: &[&str] = &["perpetual.volume24h"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerpetualDocument {
    pub id: String,
    pub perpetual: Perpetual,
    pub asset: Asset,
}

impl PerpetualDocument {
    pub fn new(perpetual: Perpetual, asset: Asset) -> Self {
        Self {
            id: sanitize_index_primary_id(&perpetual.id),
            perpetual,
            asset,
        }
    }
}

impl From<(Perpetual, Asset)> for PerpetualDocument {
    fn from((perpetual, asset): (Perpetual, Asset)) -> Self {
        Self::new(perpetual, asset)
    }
}
