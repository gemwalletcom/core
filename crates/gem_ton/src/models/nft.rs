use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct NftItemsResponse {
    #[serde(default)]
    pub nft_items: Vec<NftItem>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftCollectionsResponse {
    #[serde(default)]
    pub nft_collections: Vec<NftCollection>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftItem {
    pub address: String,
    pub collection_address: Option<String>,
    pub owner_address: Option<String>,
    pub index: Option<String>,
    #[serde(default)]
    pub content: Option<NftContent>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftCollection {
    pub address: String,
    pub owner_address: Option<String>,
    #[serde(default)]
    pub collection_content: Option<NftContent>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct NftContent {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub image_data: Option<String>,
    #[serde(default)]
    pub attributes: Vec<NftAttribute>,
    pub social_links: Option<Vec<String>>,
    pub external_url: Option<String>,
    pub external_link: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftAttribute {
    pub trait_type: Option<String>,
    pub value: Option<serde_json::Value>,
}
