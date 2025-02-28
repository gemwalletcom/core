use gem_evm::address::EthereumAddress;
use primitives::{Chain, NFTAssetId, NFTImage};

use super::model::{NFTAsset, NFTCollection, NFTCollectionAssets, NFTResult};

impl NFTResult {
    pub fn as_primitive(&self) -> Vec<NFTAssetId> {
        self.collection_assets
            .clone()
            .into_iter()
            .flat_map(|nft_collection| {
                let chain = map_chain(self.chain.as_str())?;
                if !nft_collection.is_verified() {
                    return None;
                }
                let collection_id = nft_collection.as_collection_id(chain)?;
                let assets = nft_collection
                    .assets
                    .into_iter()
                    .filter_map(|x| x.as_primitive(chain, &collection_id.contract_address))
                    .collect::<Vec<primitives::NFTAsset>>();
                Some(assets)
            })
            .flatten()
            .map(|x| x.into())
            .collect::<Vec<NFTAssetId>>()
    }
}

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

    pub fn as_primitive(&self, chain: Chain, contract_address: &str) -> Option<primitives::NFTAsset> {
        let token_type = map_erc_type(self.erc_type.as_str())?;
        let token_id = self.token_id.clone();
        let contract_address = EthereumAddress::parse(contract_address)?.to_checksum();
        let id: NFTAssetId = NFTAssetId::new(chain, &contract_address, token_id.as_str());
        let name = self.name.clone()?;

        Some(primitives::NFTAsset {
            id: id.to_string(),
            collection_id: id.get_collection_id().id(),
            contract_address: Some(contract_address.to_string()),
            token_id,
            name,
            description: self.description.clone(),
            chain,
            image: self.get_image(),
            token_type,
            attributes: self.attributes.clone().into_iter().flat_map(|x| x.as_primitive()).collect(),
        })
    }
}

impl NFTCollection {
    pub fn as_primitive(&self, chain: Chain) -> Option<primitives::NFTCollection> {
        let contract_address = EthereumAddress::parse(self.contract_address.as_str())?.to_checksum();
        let name = self.name.clone()?;
        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, contract_address.as_str()),
            name,
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

impl NFTCollectionAssets {
    pub fn as_collection_id(&self, chain: Chain) -> Option<primitives::NFTCollectionId> {
        let contract_address = EthereumAddress::parse(self.contract_address.as_str())?.to_checksum();
        Some(primitives::NFTCollectionId::new(chain, contract_address.as_str()))
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

pub fn map_erc_type(erc_type: &str) -> Option<primitives::NFTType> {
    match erc_type {
        "erc721" => Some(primitives::NFTType::ERC721),
        "erc1155" => Some(primitives::NFTType::ERC1155),
        _ => None,
    }
}
