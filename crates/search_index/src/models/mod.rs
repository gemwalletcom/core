mod asset;
mod perpetual;

pub use asset::*;
pub use perpetual::*;

use serde::{Deserialize, Serialize};

pub const INDEX_PRIMARY_KEY: &str = "id";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentId {
    pub id: String,
}

pub struct IndexConfig {
    pub name: &'static str,
    pub filters: &'static [&'static str],
    pub sorts: &'static [&'static str],
    pub search_attributes: &'static [&'static str],
    pub ranking_rules: &'static [&'static str],
}

pub const INDEX_CONFIGS: &[IndexConfig] = &[
    IndexConfig {
        name: ASSETS_INDEX_NAME,
        filters: ASSETS_FILTERS,
        sorts: ASSETS_SORTS,
        search_attributes: ASSETS_SEARCH_ATTRIBUTES,
        ranking_rules: ASSETS_RANKING_RULES,
    },
    IndexConfig {
        name: PERPETUALS_INDEX_NAME,
        filters: PERPETUALS_FILTERS,
        sorts: PERPETUALS_SORTS,
        search_attributes: PERPETUALS_SEARCH_ATTRIBUTES,
        ranking_rules: PERPETUALS_RANKING_RULES,
    },
];

pub fn sanitize_index_primary_id(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}
