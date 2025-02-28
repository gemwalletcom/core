use serde::{Deserialize, Serialize};

use crate::nftscan::model::NFTAttribute;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: usize,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTResult {
    pub chain: String,
    pub collection_assets: Vec<NFTCollectionAssets>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTCollectionAssets {
    pub token_address: Option<String>,
    pub interact_program: Option<String>,
    pub contract_name: String,
    pub contract_address: String,
    pub description: Option<String>,
    pub verified: bool,
    pub is_spam: bool,
    pub opensea_verified: bool,
    pub logo_url: Option<String>,
    pub assets: Vec<NFTAsset>,
}

impl NFTCollectionAssets {
    pub fn is_verified(&self) -> bool {
        self.opensea_verified && !self.is_spam
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTCollection {
    // pub token_address: Option<String>,
    // pub interact_program: Option<String>,
    // pub contract_name: String,
    pub contract_address: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub verified: bool,
    pub opensea_verified: bool,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAsset {
    pub token_id: String,
    pub erc_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub nftscan_uri: Option<String>,
    //pub rarity_score: Option<f64>,
    //pub rarity_rank: Option<u64>,
    //pub metadata_json: Option<String>,
    pub attributes: Vec<NFTAttribute>,
}
