use gem_evm::address::EthereumAddress;
use simplehash::client::{SIMPLEHASH_EVM_CHAINS, SIMPLEHASH_SOLANA_CHAIN};
use std::collections::HashMap;

use nftscan::{
    model::{NFTAsset, NFTCollection},
    NFTScanClient,
};
use primitives::{Chain, NFTImage};

pub mod nftscan;
pub mod opensea;
pub use opensea::OpenSeaClient;
pub mod simplehash;
pub use simplehash::SimpleHashClient;

pub struct NFT {
    nftscan_client: NFTScanClient,
    simplehash_client: SimpleHashClient,
}

impl NFT {
    pub fn new(nftscan_key: &str, simplehash_key: &str) -> Self {
        Self {
            nftscan_client: NFTScanClient::new(nftscan_key),
            simplehash_client: SimpleHashClient::new(simplehash_key.to_string()),
        }
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<primitives::NFTData>, Box<dyn std::error::Error + Send + Sync>> {
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|(chain, address)| {
                let address = address.clone();
                async move { self.get_nfts(chain, address.as_str()).await }
            })
            .collect();

        let assets = futures::future::try_join_all(futures)
            .await?
            .into_iter()
            .flatten()
            .filter(|x| x.collection.is_verified)
            .filter(|x| !x.assets.is_empty())
            .collect();

        Ok(assets)
    }

    pub async fn get_nfts(&self, chain: Chain, address: &str) -> Result<Vec<primitives::NFTData>, reqwest::Error> {
        let pages_limit = 5;
        match chain {
            Chain::Ethereum => self
                .simplehash_client
                .get_assets_all(address, SIMPLEHASH_EVM_CHAINS.to_vec(), pages_limit)
                .await
                .map(|x| x.as_primitives()),
            Chain::Ton => self.nftscan_client.get_ton_nfts(address).await.map(|x| {
                x.data
                    .into_iter()
                    .filter_map(|result| {
                        result.as_primitive().map(|collection| primitives::NFTData {
                            collection: collection.clone(),
                            assets: result.assets.into_iter().filter_map(|x| x.as_primitive(&collection.id)).collect(),
                        })
                    })
                    .collect::<Vec<_>>()
            }),
            Chain::Solana => self
                .simplehash_client
                .get_assets_all(address, SIMPLEHASH_SOLANA_CHAIN.to_vec(), pages_limit)
                .await
                .map(|x| x.as_primitives()),
            _ => Ok(vec![]),
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
}

impl NFTCollection {
    pub fn as_primitive(&self, chain: &str) -> Option<primitives::NFTCollection> {
        let chain = NFT::map_chain(chain)?;

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

impl NFTAsset {
    pub fn get_attributes(&self) -> Option<Vec<primitives::NFTAttribute>> {
        if let Some(metadata_json) = &self.metadata_json {
            let metadata: serde_json::Value = serde_json::from_str(metadata_json).ok()?;
            let attributes = metadata["attributes"]
                .as_array()?
                .iter()
                .filter_map(|attr| {
                    Some(primitives::NFTAttribute {
                        name: attr["trait_type"].as_str()?.to_string(),
                        value: attr["value"].as_str()?.to_string(),
                        percentage: None,
                    })
                })
                .collect();
            Some(attributes)
        } else {
            Some(Vec::new())
        }
    }

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
        let chain = NFT::map_chain(chain)?;
        let token_type = NFT::map_erc_type(self.erc_type.as_str())?;
        let attributes = self.get_attributes().unwrap_or_default();
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
