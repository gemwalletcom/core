use crate::chain::Chain;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFT {
    pub chain: Chain,
    pub name: String,
    #[serde(rename = "tokenAddress")]
    pub token_address: String,
    pub description: String,
    #[serde(rename = "collectionAddress")]
    pub collection_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTCollection {
    pub chain: Chain,
    pub name: String,
    #[serde(rename = "collectionAddress")]
    pub collection_address: String,
    pub description: String,
    pub nfts: Vec<NFT>,
}

impl NFT {
    pub fn new(
        chain: Chain,
        name: String,
        token_address: String,
        description: String,
        collection_address: String,
    ) -> Self {
        Self {
            chain,
            collection_address,
            description,
            name,
            token_address,
        }
    }
}
