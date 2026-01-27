use crate::{AssetBasic, NFTCollection, PerpetualSearchData};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Sendable")]
pub struct SearchResponse {
    pub assets: Vec<AssetBasic>,
    pub perpetuals: Vec<PerpetualSearchData>,
    pub nfts: Vec<NFTCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SearchItemType {
    Asset,
    Perpetual,
    Nft,
}
