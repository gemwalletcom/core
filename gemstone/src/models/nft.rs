use primitives::Chain;
use primitives::nft::{NFTAsset, NFTAttribute, NFTImages, NFTResource, NFTType};

pub type GemNFTAttribute = NFTAttribute;
pub type GemNFTType = NFTType;
pub type GemNFTResource = NFTResource;
pub type GemNFTImages = NFTImages;
pub type GemNFTAsset = NFTAsset;

#[uniffi::remote(Enum)]
pub enum GemNFTType {
    ERC721,
    ERC1155,
    SPL,
    JETTON,
}

#[uniffi::remote(Record)]
pub struct GemNFTResource {
    pub url: String,
    pub mime_type: String,
}

#[uniffi::remote(Record)]
pub struct GemNFTImages {
    pub preview: GemNFTResource,
}

#[uniffi::remote(Record)]
pub struct GemNFTAttribute {
    pub name: String,
    pub value: String,
    pub percentage: Option<f64>,
}

#[uniffi::remote(Record)]
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
