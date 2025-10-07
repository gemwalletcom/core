use primitives::NFTCollection;
use serde::{Deserialize, Serialize};

use crate::sanitize_index_primary_id;

pub const NFTS_INDEX_NAME: &str = "nfts";
pub const NFTS_FILTERS: &[&str] = &["collection.chain", "collection.name", "collection.contractAddress", "collection.isVerified"];
pub const NFTS_SEARCH_ATTRIBUTES: &[&str] = &["collection.name", "collection.contractAddress", "collection.chain"];
pub const NFTS_RANKING_RULES: &[&str] = &["words", "typo", "proximity", "attribute", "exactness"];

pub const NFTS_SORTS: &[&str] = &[];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFTDocument {
    pub id: String,
    pub collection: NFTCollection,
}

impl NFTDocument {
    pub fn new(collection: NFTCollection) -> Self {
        Self {
            id: sanitize_index_primary_id(&collection.id),
            collection,
        }
    }
}

impl From<NFTCollection> for NFTDocument {
    fn from(collection: NFTCollection) -> Self {
        Self::new(collection)
    }
}
