use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Contract {
    pub address: String,
    pub chain: String,
    pub collection: String,
    pub contract_standard: String,
    pub name: String,
    pub total_supply: Option<u64>,
}

#[derive(Deserialize)]
pub struct Collection {
    pub collection: String,
    pub name: String,
    pub description: String,
    // pub image_url: String,
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

// #[derive(Deserialize)]
// pub struct Fee {
//     pub fee: f64,
//     pub recipient: String,
//     pub required: bool,
// }

// #[derive(Deserialize)]
// pub struct Rarity {
//     pub strategy_id: String,
//     pub strategy_version: String,
//     pub calculated_at: String,
//     pub max_rank: u32,
//     pub tokens_scored: u32,
// }
