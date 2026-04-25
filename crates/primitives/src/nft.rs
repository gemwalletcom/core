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

use crate::{AssetLink, CHAIN_SEPARATOR, Chain, TOKEN_ID_SEPARATOR, VerificationStatus};

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
    // TODO: Remove after all Rust callers and downstream indexes migrate to `status`.
    #[serde(default)]
    #[typeshare(skip)]
    pub is_verified: bool,
    pub status: VerificationStatus,
    pub links: Vec<AssetLink>,
}

impl Hash for NFTCollection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl NFTCollection {
    pub fn id(chain: Chain, contract_address: &str) -> String {
        format!("{}{CHAIN_SEPARATOR}{}", chain.as_ref(), contract_address)
    }

    pub fn images(&self) -> NFTImages {
        let image = format!("{}/{}/collection_original.png", self.chain.as_ref(), self.contract_address);
        NFTImages {
            preview: NFTResource::from_url(&image),
        }
    }

    pub fn with_preview_url(self, url: String) -> Self {
        Self {
            images: NFTImages {
                preview: NFTResource { url, ..self.images.preview },
            },
            ..self
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

impl NFTAsset {
    pub fn get_contract_address(&self) -> Result<&str, &'static str> {
        self.contract_address.as_deref().ok_or("missing NFT contract address")
    }

    pub fn with_urls(self, preview_url: String, resource_url: String) -> Self {
        Self {
            images: NFTImages {
                preview: NFTResource {
                    url: preview_url,
                    ..self.images.preview
                },
            },
            resource: NFTResource {
                url: resource_url,
                ..self.resource
            },
            ..self
        }
    }
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

    pub fn from_id(id: &str) -> Option<Self> {
        let (chain, contract_address) = id.split_once(CHAIN_SEPARATOR)?;
        Some(Self {
            chain: Chain::from_str(chain).ok()?,
            contract_address: contract_address.to_string(),
        })
    }

    pub fn id(&self) -> String {
        format!("{}{CHAIN_SEPARATOR}{}", self.chain.as_ref(), self.contract_address)
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
        let (chain, rest) = id.split_once(CHAIN_SEPARATOR)?;
        let (contract_address, token_id) = rest.split_once(TOKEN_ID_SEPARATOR)?;
        Some(Self {
            chain: Chain::from_str(chain).ok()?,
            contract_address: contract_address.to_string(),
            token_id: token_id.to_string(),
        })
    }

    pub fn get_collection_id(&self) -> NFTCollectionId {
        NFTCollectionId::new(self.chain, &self.contract_address)
    }
}

impl AsRef<str> for NFTAssetId {
    fn as_ref(&self) -> &str {
        Box::leak(format!("{}{CHAIN_SEPARATOR}{}{TOKEN_ID_SEPARATOR}{}", self.chain.as_ref(), self.contract_address, self.token_id).into_boxed_str())
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumIter, AsRefStr, EnumString)]
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

#[derive(Debug, Serialize, Deserialize, Clone, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Sendable, CaseIterable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReportReason {
    Spam,
    Malicious,
    Inappropriate,
    Copyright,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub struct ReportNft {
    #[typeshare(skip)]
    pub device_id: String,
    pub collection_id: String,
    pub asset_id: Option<String>,
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TON_COLLECTION: &str = "EQC3dNlesgVD8YbAazcauIrXBPfiVhMMr5YYk2in0Mtsz0Bz";
    const TON_TOKEN: &str = "EQAqmedq_nTBz7rX6TvASY_kwXxbKexQap_qnsfS4E-qF0dI";

    #[test]
    fn test_collection_id() {
        let eth = NFTCollectionId::new(Chain::Ethereum, "0xabc");
        assert_eq!(eth.id(), "ethereum_0xabc");
        assert_eq!(NFTCollectionId::from_id(&eth.id()), Some(eth));

        let ton = NFTCollectionId::new(Chain::Ton, TON_COLLECTION);
        assert_eq!(ton.id(), format!("ton_{TON_COLLECTION}"));
        assert_eq!(NFTCollectionId::from_id(&ton.id()), Some(ton));

        assert_eq!(NFTCollectionId::from_id("just-chain"), None);
        assert_eq!(NFTCollectionId::from_id("not_a_real_chain"), None);
    }

    #[test]
    fn test_asset_id() {
        let eth = NFTAssetId::new(Chain::Ethereum, "0xabc", "42");
        assert_eq!(eth.as_ref(), "ethereum_0xabc::42");
        assert_eq!(NFTAssetId::from_id("ethereum_0xabc::42"), Some(eth));

        let ton = NFTAssetId::new(Chain::Ton, TON_COLLECTION, TON_TOKEN);
        assert_eq!(ton.as_ref(), format!("ton_{TON_COLLECTION}::{TON_TOKEN}"));
        assert_eq!(NFTAssetId::from_id(ton.as_ref()), Some(ton.clone()));
        assert_eq!(ton.get_collection_id(), NFTCollectionId::new(Chain::Ton, TON_COLLECTION));

        assert_eq!(NFTAssetId::from_id("ethereum_0xabc"), None);
        assert_eq!(NFTAssetId::from_id("nonsense"), None);
        assert_eq!(NFTAssetId::from_id("ton_short"), None);
    }
}
