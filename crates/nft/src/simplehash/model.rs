use serde::{Deserialize, Serialize};

pub(crate) const MARKET_OPENSEA_ID: &str = "opensea";
pub(crate) const MARKET_MAGICEDEN_ID: &str = "magiceden";

#[derive(Debug, Serialize, Deserialize)]
pub struct NftResponse {
    pub next_cursor: Option<String>,
    pub nfts: Vec<Nft>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nft {
    pub chain: String,
    pub contract_address: String,
    pub token_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub previews: Previews,
    //     pub image_url: String,
    //pub image_properties: ImageProperties,
    //     pub video_url: Option<String>,
    //     pub video_properties: Option<MediaProperties>,
    //     pub audio_url: Option<String>,
    //     pub audio_properties: Option<MediaProperties>,
    //     pub model_url: Option<String>,
    //     pub model_properties: Option<MediaProperties>,
    //     pub other_url: Option<String>,
    //     pub other_properties: Option<MediaProperties>,
    //     pub background_color: Option<String>,
    //     pub external_url: Option<String>,
    //     pub created_date: String,
    //     pub status: String,
    //     pub token_count: u32,
    //     pub owner_count: u32,
    //     pub owners: Vec<Owner>,
    pub contract: Contract,
    pub collection: Collection,
    //     pub last_sale: Option<Sale>,
    //     pub primary_sale: Option<Sale>,
    //     pub first_created: FirstCreated,
    //     pub rarity: Rarity,
    //     pub royalty: Vec<Royalty>,
    pub extra_metadata: ExtraMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Previews {
    pub image_small_url: Option<String>,
    pub image_medium_url: Option<String>,
    pub image_large_url: Option<String>,
    //pub image_opengraph_url: String,
    //pub blurhash: String,
    //pub predominant_color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageProperties {
    // pub width: u32,
    // pub height: u32,
    // pub size: u32,
    pub mime_type: Option<String>,
    // pub exif_orientation: Option<String>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct MediaProperties {
//     pub width: Option<u32>,
//     pub height: Option<u32>,
//     pub size: Option<u32>,
//     pub mime_type: Option<String>,
//     pub exif_orientation: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Owner {
//     pub owner_address: String,
//     pub quantity: u32,
//     pub quantity_string: String,
//     pub first_acquired_date: String,
//     pub last_acquired_date: String,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Contract {
    #[serde(rename = "type")]
    pub contract_type: Option<String>,
    //     pub name: String,
    //     pub symbol: String,
    //     pub deployed_by: String,
    //     pub deployed_via_contract: Option<String>,
    //     pub owned_by: String,
    //     pub has_multiple_collections: bool,
    //     pub has_erc5643_subscription_standard: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    //     pub collection_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub image_properties: Option<ImageProperties>,
    //     pub banner_image_url: String,
    //     pub category: String,
    //     pub is_nsfw: bool,
    //     pub external_url: String,
    //     pub twitter_username: String,
    //     pub discord_url: String,
    //     pub instagram_username: String,
    //     pub medium_username: Option<String>,
    //     pub telegram_url: Option<String>,
    pub marketplace_pages: Vec<MarketplacePage>,
    pub metaplex_mint: Option<String>,
    pub metaplex_candy_machine: Option<String>,
    //     pub metaplex_first_verified_creator: Option<String>,
    //     pub mpl_core_collection_address: Option<String>,
    //     pub floor_prices: Vec<FloorPrice>,
    //     pub top_bids: Vec<Bid>,
    //     pub distinct_owner_count: u32,
    //     pub distinct_nft_count: u32,
    //     pub total_quantity: u32,
    //     pub chains: Vec<String>,
    //     pub top_contracts: Vec<String>,
    //     pub collection_royalties: Vec<Royalty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplacePage {
    pub marketplace_id: String,
    //pub marketplace_name: String,
    //pub marketplace_collection_id: String,
    //pub nft_url: String,
    //pub collection_url: String,
    pub verified: Option<bool>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct FloorPrice {
//     pub marketplace_id: String,
//     pub marketplace_name: String,
//     pub value: u64,
//     pub payment_token: PaymentToken,
//     pub value_usd_cents: u32,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Bid {
//     pub marketplace_id: String,
//     pub marketplace_name: String,
//     pub value: u64,
//     pub payment_token: PaymentToken,
//     pub value_usd_cents: u32,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct PaymentToken {
//     pub payment_token_id: String,
//     pub name: String,
//     pub symbol: String,
//     pub address: Option<String>,
//     pub decimals: u8,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Sale {
//     pub from_address: Option<String>,
//     pub to_address: String,
//     pub quantity: u32,
//     pub quantity_string: String,
//     pub timestamp: String,
//     pub transaction: String,
//     pub marketplace_id: String,
//     pub marketplace_name: String,
//     pub is_bundle_sale: bool,
//     pub payment_token: PaymentToken,
//     pub unit_price: u64,
//     pub total_price: u64,
//     pub unit_price_usd_cents: u32,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct FirstCreated {
//     pub minted_to: String,
//     pub quantity: u32,
//     pub quantity_string: String,
//     pub timestamp: String,
//     pub block_number: u64,
//     pub transaction: String,
//     pub transaction_initiator: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Rarity {
//     pub rank: u32,
//     pub score: f64,
//     pub unique_attributes: u32,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Royalty {
//     pub source: String,
//     pub total_creator_fee_basis_points: u32,
//     pub recipients: Vec<RoyaltyRecipient>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct RoyaltyRecipient {
//     pub address: String,
//     pub percentage: f64,
//     pub basis_points: u32,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtraMetadata {
    pub attributes: Vec<Attribute>,
    //pub image_original_url: String,
    //pub animation_original_url: Option<String>,
    //pub metadata_original_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
    //pub display_type: Option<String>,
}
