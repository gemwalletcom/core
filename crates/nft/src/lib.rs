use std::collections::HashMap;

use nftscan::{
    model::{NFTAsset, NFTCollection},
    NFTScanClient,
};
use primitives::{Chain, ChainType, NFTImage};

mod nftscan;

pub struct NFT {
    client: NFTScanClient,
}

impl NFT {
    pub fn new(nftscan_key: &str) -> Self {
        Self {
            client: NFTScanClient::new(nftscan_key),
        }
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<primitives::NFTData>, Box<dyn std::error::Error>> {
        //TODO: Add support for Solana/Ton and other chains
        let evm_address = addresses.into_iter().find(|x| x.0.chain_type() == ChainType::Ethereum).unwrap().1;
        let result = self.client.get_all_evm_nfts(evm_address.as_str()).await?.data;

        let assets = result
            .into_iter()
            .flat_map(|result| {
                result.collection_assets.into_iter().filter_map(move |x| {
                    x.as_primitive(&result.chain).map(|collection| primitives::NFTData {
                        collection: collection.clone(),
                        assets: x.assets.into_iter().filter_map(|x| x.as_primitive(&result.chain, &collection.id)).collect(),
                    })
                })
            })
            .filter(|x| x.collection.is_verified)
            .collect();

        Ok(assets)
    }

    pub fn map_chain(chain: &str) -> Option<primitives::Chain> {
        match chain {
            "eth" => Some(Chain::Ethereum),
            "base" => Some(Chain::Base),
            "bnb" => Some(Chain::SmartChain),
            "polygon" => Some(Chain::Polygon),
            "arbitrum" => Some(Chain::Arbitrum),
            _ => None,
        }
    }

    pub fn map_erc_type(map_erc_type: &str) -> Option<primitives::NFTType> {
        match map_erc_type {
            "erc721" => Some(primitives::NFTType::ERC721),
            "erc1155" => Some(primitives::NFTType::ERC1155),
            _ => None,
        }
    }
}

impl NFTCollection {
    pub fn as_primitive(&self, chain: &str) -> Option<primitives::NFTCollection> {
        let chain = NFT::map_chain(chain)?;

        Some(primitives::NFTCollection {
            id: self.contract_address.to_string(),
            name: self.contract_name.to_string(),
            description: self.description.clone(),
            chain,
            image: NFTImage {
                image_url: self.logo_url.clone().unwrap_or_default(),
                preview_image_url: self.logo_url.clone().unwrap_or_default(),
                original_source_url: self.logo_url.clone().unwrap_or_default(),
            },
            is_verified: self.opensea_verified || self.verified,
        })
    }
}
impl NFTAsset {
    pub fn as_primitive(&self, chain: &str, collection_id: &str) -> Option<primitives::NFTAsset> {
        let chain = NFT::map_chain(chain)?;
        let collectible_type = NFT::map_erc_type(self.erc_type.as_str())?;

        Some(primitives::NFTAsset {
            id: self.token_id.to_string(),
            collection_id: collection_id.to_string(),
            name: self.name.to_string(),
            description: self.description.clone(),
            chain,
            image: NFTImage {
                image_url: self.image_uri.clone().unwrap_or_default(),
                preview_image_url: self.image_uri.clone().unwrap_or_default(),
                original_source_url: self.image_uri.clone().unwrap_or_default(),
            },
            collectible_type,
            attributes: self
                .attributes
                .clone()
                .into_iter()
                .map(|x| primitives::NFTAttrubute {
                    name: x.attribute_name,
                    value: x.attribute_value,
                })
                .collect(),
        })
    }
}
