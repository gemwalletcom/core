use std::fmt;
use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

pub const MIME_TYPE_PNG: &str = "image/png";
pub const MIME_TYPE_JPG: &str = "image/jpeg";
pub const MIME_TYPE_SVG: &str = "image/svg+xml";

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
    #[typeshare(skip)]
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub chain: Chain,
    pub contract_address: String,
    pub images: NFTImages,
    pub is_verified: bool,
    pub links: Vec<AssetLink>,
}

impl Hash for NFTCollection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl NFTCollection {
    pub fn id(chain: Chain, contract_address: &str) -> String {
        format!("{}_{}", chain.as_ref(), contract_address)
    }

    pub fn images(&self) -> NFTImages {
        let image = format!("{}/{}/collection_original.png", self.chain.as_ref(), self.contract_address);
        NFTImages {
            preview: NFTResource::from_url(&image),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable, Identifiable")]
pub struct NFTAsset {
    pub id: String,
    pub collection_id: String,
    pub contract_address: Option<String>,
    pub token_id: String,
    pub token_type: NFTType,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub resource: NFTResource,
    pub images: NFTImages,
    pub attributes: Vec<NFTAttribute>,
}

impl From<NFTAsset> for NFTAssetId {
    fn from(asset: NFTAsset) -> Self {
        NFTAssetId::new(asset.chain, asset.contract_address.as_deref().unwrap_or_default(), &asset.token_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTAssetData {
    pub collection: NFTCollection,
    pub asset: NFTAsset,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct NFTAssetId {
    pub chain: Chain,
    pub contract_address: String,
    pub token_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct NFTCollectionId {
    pub chain: Chain,
    pub contract_address: String,
}

impl NFTCollectionId {
    pub fn new(chain: Chain, contract_address: &str) -> Self {
        Self {
            chain,
            contract_address: contract_address.to_string(),
        }
    }

    pub fn id(&self) -> String {
        format!("{}_{}", self.chain.as_ref(), self.contract_address)
    }
}

impl NFTAssetId {
    pub fn new(chain: Chain, contract_address: &str, token_id: &str) -> Self {
        Self {
            chain,
            contract_address: contract_address.to_string(),
            token_id: token_id.to_string(),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        let parts: Vec<&str> = id.split('_').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            chain: Chain::from_str(parts[0]).ok()?,
            contract_address: parts[1].to_string(),
            token_id: parts[2].to_string(),
        })
    }

    pub fn get_collection_id(&self) -> NFTCollectionId {
        NFTCollectionId::new(self.chain, &self.contract_address)
    }
}

impl AsRef<str> for NFTAssetId {
    fn as_ref(&self) -> &str {
        Box::leak(format!("{}_{}_{}", self.chain.as_ref(), self.contract_address, self.token_id).into_boxed_str())
    }
}

impl fmt::Display for NFTAssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTResource {
    pub url: String,
    pub mime_type: String,
}

impl NFTResource {
    pub fn new(url: String, mime_type: String) -> Self {
        Self { url, mime_type }
    }

    pub fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            mime_type: mime_type_for_image_url(url),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
pub struct NFTImages {
    pub preview: NFTResource,
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
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}

fn mime_type_for_image_url(url: &str) -> String {
    if url.ends_with(".jpeg") || url.ends_with(".jpg") {
        MIME_TYPE_JPG.to_string()
    } else if url.ends_with(".svg") {
        MIME_TYPE_SVG.to_string()
    } else {
        MIME_TYPE_PNG.to_string()
    }
}
