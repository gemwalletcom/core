use gem_evm::address::EthereumAddress;
use primitives::{Chain, NFTImage};

use super::model::{NFTAsset, NFTCollection};

impl NFTAsset {
    pub fn get_image(&self) -> NFTImage {
        let image_url = self.image_uri.clone().unwrap_or_default();

        if let Some(image_url) = self.nftscan_uri.clone() {
            return NFTImage {
                image_url: image_url.clone(),
                preview_image_url: image_url.clone(),
                original_source_url: image_url.clone(),
            };
        }

        let image_url = if image_url.clone().starts_with("Qm") {
            format!("https://ipfs.io/ipfs/{}", image_url)
        } else {
            image_url
        };
        NFTImage {
            image_url: image_url.clone(),
            preview_image_url: image_url.clone(),
            original_source_url: image_url.clone(),
        }
    }

    pub fn as_primitive(&self, chain: &str, contract_address: &str) -> Option<primitives::NFTAsset> {
        let chain = map_chain(chain)?;
        let token_type = map_erc_type(self.erc_type.as_str())?;
        let attributes = vec![];
        let token_id = self.token_id.clone();

        Some(primitives::NFTAsset {
            id: primitives::NFTAsset::id(chain, contract_address, token_id.as_str()),
            collection_id: primitives::NFTCollection::id(chain, contract_address),
            contract_address: Some(contract_address.to_string()),
            token_id,
            name: self.name.clone().unwrap_or_default().to_string(),
            description: self.description.clone(),
            chain,
            image: self.get_image(),
            token_type,
            attributes,
        })
    }
}

impl NFTCollection {
    pub fn as_primitive(&self, chain: &str) -> Option<primitives::NFTCollection> {
        let chain = map_chain(chain)?;

        let contract_address = Some(EthereumAddress::parse(self.contract_address.as_str())?.to_checksum())?;

        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, contract_address.as_str()),
            name: self.contract_name.clone().to_string(),
            description: self.description.clone(),
            chain,
            contract_address: contract_address.clone(),
            image: NFTImage {
                image_url: self.logo_url.clone().unwrap_or_default(),
                preview_image_url: self.logo_url.clone().unwrap_or_default(),
                original_source_url: self.logo_url.clone().unwrap_or_default(),
            },
            is_verified: self.opensea_verified || self.verified,
            links: vec![],
        })
    }
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
