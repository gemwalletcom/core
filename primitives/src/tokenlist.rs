use typeshare::typeshare;
use crate::chain::Chain;
use crate::asset_type::AssetType;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TokenList {
    pub version: i32,
    pub assets: Vec<TokenListAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TokenListAsset {
    pub chain: Chain,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    pub decimals: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TokenListChainVersion {
    pub chain: String,
    pub version: i32,
}