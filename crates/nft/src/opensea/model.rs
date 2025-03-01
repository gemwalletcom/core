use serde::Deserialize;

pub type TokenStandard = String;

#[derive(Deserialize, Debug)]
pub struct Contract {
    pub address: String,
    pub chain: String,
    pub collection: String,
    pub contract_standard: String,
    pub name: String,
    pub total_supply: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct NftsResponse {
    pub nfts: Vec<NftAsset>,
}

#[derive(Deserialize, Debug)]
pub struct NftResponse {
    pub nft: Nft,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftAsset {
    pub identifier: String,
    pub contract: String,
    pub token_standard: TokenStandard,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Nft {
    pub identifier: String,
    pub collection: String,
    pub contract: String,
    pub token_standard: TokenStandard,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub display_image_url: String,
    pub traits: Option<Vec<Trait>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trait {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Deserialize)]
pub struct Collection {
    pub collection: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
    // pub banner_image_url: String,
    // pub owner: String,
    // pub safelist_status: String,
    // pub category: String,
    // pub is_disabled: bool,
    // pub is_nsfw: bool,
    // pub trait_offers_enabled: bool,
    // pub collection_offers_enabled: bool,
    pub opensea_url: String,
    pub project_url: String,
    // pub wiki_url: String,
    pub discord_url: String,
    pub telegram_url: String,
    pub twitter_username: String,
    pub instagram_username: String,
    // pub editors: Vec<String>,
    // pub fees: Vec<Fee>,
    // pub rarity: Rarity,
    // pub total_supply: u32,
    // pub created_date: String,
}
