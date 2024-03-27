use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Codable")]
pub struct NFTCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub chain: Chain,
    pub image: NFTImage,
    pub explorer_url: String,
    pub count: i64, // number of collectibles inside a collection (for specific address)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Codable")]
pub struct NFTCollectible {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub description: String,
    pub chain: Chain,
    pub image: NFTImage,
    #[serde(rename = "type")]
    pub collectible_type: NFTType,
    pub attributes: Vec<NFTAttrubute>,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Codable")]
pub struct NFTImage {
    pub image_url: String,
    pub preview_image_url: String,
    pub original_source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Codable")]
pub struct NFTAttrubute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsRefStr, EnumString)]
#[typeshare(swift = "Codable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NFTType {
    ERC721,
    ERC1155,
    SPL,
    TON,
}
