use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::Chain;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct NFTCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub image: NFTImage,
    pub is_verified: bool,
    pub assets: Vec<NFTAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct NFTAsset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub image: NFTImage,
    #[serde(rename = "type")]
    pub collectible_type: NFTType,
    pub attributes: Vec<NFTAttrubute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct NFTImage {
    pub image_url: String,
    pub preview_image_url: String,
    pub original_source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct NFTAttrubute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, AsRefStr, EnumString)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NFTType {
    ERC721,
    ERC1155,
    SPL,
}
