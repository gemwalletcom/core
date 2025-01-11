use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: usize,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTResult {
    pub chain: String,
    pub collection_assets: Vec<NFTCollection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTCollection {
    pub contract_name: String,
    pub contract_address: String,
    pub description: Option<String>,
    pub verified: bool,
    pub opensea_verified: bool,
    pub logo_url: Option<String>,
    pub assets: Vec<NFTAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTAsset {
    pub token_id: String,
    pub erc_type: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: String,
    pub image_uri: Option<String>,
}
