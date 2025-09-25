use primitives::Chain;
use primitives::nft::{NFTAsset, NFTAttribute, NFTImages, NFTResource, NFTType};

pub type GemNFTAttribute = NFTAttribute;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemNFTType {
    Erc721,
    Erc1155,
    Spl,
    Jetton,
}

impl From<NFTType> for GemNFTType {
    fn from(value: NFTType) -> Self {
        match value {
            NFTType::ERC721 => GemNFTType::Erc721,
            NFTType::ERC1155 => GemNFTType::Erc1155,
            NFTType::SPL => GemNFTType::Spl,
            NFTType::JETTON => GemNFTType::Jetton,
        }
    }
}

impl From<GemNFTType> for NFTType {
    fn from(value: GemNFTType) -> Self {
        match value {
            GemNFTType::Erc721 => NFTType::ERC721,
            GemNFTType::Erc1155 => NFTType::ERC1155,
            GemNFTType::Spl => NFTType::SPL,
            GemNFTType::Jetton => NFTType::JETTON,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNFTResource {
    pub url: String,
    pub mime_type: String,
}

impl From<NFTResource> for GemNFTResource {
    fn from(value: NFTResource) -> Self {
        GemNFTResource {
            url: value.url,
            mime_type: value.mime_type,
        }
    }
}

impl From<GemNFTResource> for NFTResource {
    fn from(value: GemNFTResource) -> Self {
        NFTResource {
            url: value.url,
            mime_type: value.mime_type,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNFTImages {
    pub preview: GemNFTResource,
}

impl From<NFTImages> for GemNFTImages {
    fn from(value: NFTImages) -> Self {
        GemNFTImages { preview: value.preview.into() }
    }
}

impl From<GemNFTImages> for NFTImages {
    fn from(value: GemNFTImages) -> Self {
        NFTImages { preview: value.preview.into() }
    }
}

#[uniffi::remote(Record)]
pub struct GemNFTAttribute {
    pub name: String,
    pub value: String,
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNFTAsset {
    pub id: String,
    pub collection_id: String,
    pub contract_address: Option<String>,
    pub token_id: String,
    pub token_type: GemNFTType,
    pub name: String,
    pub description: Option<String>,
    pub chain: Chain,
    pub resource: GemNFTResource,
    pub images: GemNFTImages,
    pub attributes: Vec<GemNFTAttribute>,
}

impl From<NFTAsset> for GemNFTAsset {
    fn from(value: NFTAsset) -> Self {
        GemNFTAsset {
            id: value.id,
            collection_id: value.collection_id,
            contract_address: value.contract_address,
            token_id: value.token_id,
            token_type: value.token_type.into(),
            name: value.name,
            description: value.description,
            chain: value.chain,
            resource: value.resource.into(),
            images: value.images.into(),
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<GemNFTAsset> for NFTAsset {
    fn from(value: GemNFTAsset) -> Self {
        NFTAsset {
            id: value.id,
            collection_id: value.collection_id,
            contract_address: value.contract_address,
            token_id: value.token_id,
            token_type: value.token_type.into(),
            name: value.name,
            description: value.description,
            chain: value.chain,
            resource: value.resource.into(),
            images: value.images.into(),
            attributes: value.attributes.into_iter().map(Into::into).collect(),
        }
    }
}
