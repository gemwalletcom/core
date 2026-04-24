use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct NftItemsResponse {
    pub nft_items: Vec<NftItem>,
    pub metadata: HashMap<String, TokenMetadata>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftCollectionsResponse {
    pub nft_collections: Vec<NftCollection>,
    pub metadata: HashMap<String, TokenMetadata>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TokenMetadata {
    pub token_info: Vec<TokenInfo>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TokenInfo {
    pub valid: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub extra: Option<TokenInfoExtra>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TokenInfoExtra {
    pub domain: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftItem {
    pub address: String,
    pub collection_address: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftCollection {
    pub address: String,
}
