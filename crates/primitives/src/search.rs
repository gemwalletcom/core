use crate::{AssetBasic, NFTCollection, Perpetual};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub assets: Vec<AssetBasic>,
    pub perpetuals: Vec<Perpetual>,
    pub nfts: Vec<NFTCollection>,
}
