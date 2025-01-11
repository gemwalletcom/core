use primitives::{Asset, AssetProperties, AssetScore};
use serde::{Deserialize, Serialize};

pub const ASSETS_INDEX_NAME: &str = "assets";
pub const ASSETS_FILTERS: &[&str] = &[
    "asset.id.chain",
    "asset.id.tokenId",
    "asset.name",
    "asset.symbol",
    "asset.type",
    "score.rank",
    "properties.isEnabled",
];
pub const ASSETS_SEARCH_ATTRIBUTES: &[&str] = &["asset.id.tokenId", "asset.id.chain", "asset.name", "asset.symbol", "asset.type"];
pub const ASSETS_RANKING_RULES: &[&str] = &["words", "typo", "score.rank:desc", "proximity", "attribute", "exactness"];

pub const ASSETS_SORTS: &[&str] = &["score.rank"];

pub const INDEX_PRIMARY_KEY: &str = "id";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetDocument {
    pub id: String,
    pub asset: Asset,
    pub properties: AssetProperties,
    pub score: AssetScore,
    //TODO: Add price (market cap / supply and other metrics)
}

pub fn sanitize_index_primary_id(input: &str) -> String {
    input
        .chars() // Iterate over each character
        .filter(|c| c.is_ascii_alphanumeric())
        .collect() // Collect the result into a String
}
