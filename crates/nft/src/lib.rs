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

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<primitives::NFTCollection>, Box<dyn std::error::Error>> {
        //TODO: Add support for Solana/Ton and other chains
        let evm_address = addresses.into_iter().find(|x| x.0.chain_type() == ChainType::Ethereum).unwrap().1;
        let result = self.client.get_all_evm_nfts(evm_address.as_str()).await?.data;

        let assets = result
            .into_iter()
            .flat_map(|result| {
                result
                    .collection_assets
                    .into_iter()
                    .filter_map(move |x| x.as_primitive(&result.chain, x.assets.clone().into_iter().filter_map(|x| x.as_primitive()).collect()))
            })
            .filter(|x| x.is_verified)
            .collect();

        Ok(assets)
    }
}

impl NFTCollection {
    pub fn as_primitive(&self, chain: &str, assets: Vec<primitives::NFTAsset>) -> Option<primitives::NFTCollection> {
        let chain = match chain {
            "eth" => Chain::Ethereum,
            "base" => Chain::Base,
            "bnb" => Chain::SmartChain,
            "polygon" => Chain::Polygon,
            "arbitrum" => Chain::Arbitrum,
            _ => return None,
        };

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
            assets,
        })
    }
}
impl NFTAsset {
    pub fn as_primitive(&self) -> Option<primitives::NFTAsset> {
        let collectible_type = match self.erc_type.as_str() {
            "erc721" => primitives::nft::NFTType::ERC721,
            "erc1155" => primitives::nft::NFTType::ERC1155,
            _ => return None,
        };

        Some(primitives::NFTAsset {
            id: self.token_id.to_string(),
            name: self.name.to_string(),
            description: self.description.clone(),
            chain: Chain::Ethereum,
            image: NFTImage {
                image_url: self.image_uri.clone().unwrap_or_default(),
                preview_image_url: self.image_uri.clone().unwrap_or_default(),
                original_source_url: self.image_uri.clone().unwrap_or_default(),
            },
            collectible_type,
            attributes: vec![],
        })
    }
}
