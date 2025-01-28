use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{AssetLink, Chain};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTData {
    pub collection: NFTCollection,
    pub assets: Vec<NFTAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable, Identifiable")]
pub struct NFTCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub contract_address: String,
    pub image: NFTImage,
    pub is_verified: bool,
    pub links: Vec<AssetLink>,
}

impl Hash for NFTCollection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl NFTCollection {
    pub fn id(chain: Chain, contract_id: &str) -> String {
        format!("{}_{}", chain.as_ref(), contract_id)
    }

    pub fn image_path(&self) -> NFTImage {
        let image = format!("{}/{}/collection_original.png", self.chain.as_ref(), self.contract_address);
        NFTImage {
            image_url: image.clone(),
            preview_image_url: image.clone(),
            original_source_url: image.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable, Identifiable")]
pub struct NFTAsset {
    pub id: String,
    pub collection_id: String,
    pub token_id: String,
    pub token_type: NFTType,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub image: NFTImage,
    pub attributes: Vec<NFTAttribute>,
}

pub struct NFTAssetId {
    pub chain: Chain,
    pub collection_id: String,
    pub token_id: String,
}

impl NFTAssetId {
    pub fn from_id(id: &str) -> Self {
        let parts: Vec<&str> = id.split('_').collect();
        Self {
            chain: Chain::from_str(parts[0]).unwrap(),
            collection_id: parts[1].to_string(),
            token_id: parts[2].to_string(),
        }
    }
}

impl NFTAsset {
    pub fn id(collection_id: &str, token_id: &str) -> String {
        format!("{}_{}", collection_id, token_id)
    }

    pub fn image_path(&self) -> NFTImage {
        let asset_id = NFTAssetId::from_id(self.id.clone().as_str());
        let image = format!("{}/{}/assets/{}_original.png", self.chain.as_ref(), asset_id.collection_id, self.token_id);
        NFTImage {
            image_url: image.clone(),
            preview_image_url: image.clone(),
            original_source_url: image.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTImage {
    pub image_url: String,
    pub preview_image_url: String,
    pub original_source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTAttribute {
    pub name: String,
    pub value: String,
    pub percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NFTType {
    ERC721,
    ERC1155,
    SPL,
    JETTON,
}

impl NFTType {
    pub fn all() -> Vec<NFTType> {
        NFTType::iter().collect::<Vec<_>>()
    }
}
