use crate::Asset;
use crate::{chain::Chain, AssetId};
use crate::asset_type::AssetType;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenList {
    pub version: i32,
    pub assets: Vec<TokenListAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl TokenListAsset {
    pub fn to_asset(&self) -> Asset {
        Asset {
            id: AssetId {
                chain: self.chain,
                token_id: Some(self.token_id.clone()),
            },
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            asset_type: self.asset_type.clone(),
            decimals: self.decimals,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListChainVersion {
    pub chain: String,
    pub version: i32,
}