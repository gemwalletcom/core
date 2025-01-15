use gem_evm::address::EthereumAddress;
use gem_solana::TOKEN_PROGRAM;
use std::collections::HashMap;

use nftscan::{
    model::{NFTAsset, NFTAttribute, NFTCollection, NFTSolanaAsset, NFTSolanaResult},
    NFTScanClient,
};
use primitives::{Chain, NFTImage};

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
            .collect();

        Ok(assets)
    }

    pub async fn get_nfts(&self, chain: Chain, address: &str) -> Result<Vec<primitives::NFTData>, reqwest::Error> {
        match chain {
            Chain::Ethereum => self.client.get_all_evm_nfts(address).await.map(|x| x.data).map(|result| {
                result
                    .into_iter()
                    .flat_map(|result| {
                        result.collection_assets.into_iter().filter_map(move |x| {
                            x.as_primitive(&result.chain).map(|collection| primitives::NFTData {
                                collection: collection.clone(),
                                assets: x.assets.into_iter().filter_map(|x| x.as_primitive(&result.chain, &collection.id)).collect(),
                            })
                        })
                    })
                    .collect::<Vec<_>>()
            }),
            Chain::Solana => self.client.get_solana_nfts(address).await.map(|x| {
                x.data
                    .into_iter()
                    .filter_map(|result| {
                        result.as_primitive(address).map(|collection| primitives::NFTData {
                            collection: collection.clone(),
                            assets: result.assets.into_iter().filter_map(|x| x.as_primitive(&collection.id)).collect(),
                        })
                    })
                    .filter(|x| !x.assets.is_empty())
                    .collect::<Vec<_>>()
            }),
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
            name: self.contract_name.to_string(),
            description: self.description.clone(),
            chain,
            contract_address: contract_address.clone(),
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

    pub fn as_primitive(&self, chain: &str, collection_id: &str) -> Option<primitives::NFTAsset> {
        let chain = NFT::map_chain(chain)?;
        let token_type = NFT::map_erc_type(self.erc_type.as_str())?;
        let attributes = self.get_attributes().unwrap_or_default();
        let token_id = self.token_id.clone();

        Some(primitives::NFTAsset {
            id: primitives::NFTAsset::id(collection_id, token_id.as_str()),
            collection_id: collection_id.to_string(),
            token_id,
            name: self.name.to_string(),
            description: self.description.clone(),
            chain,
            image: self.get_image(),
            token_type,
            attributes,
        })
    }
}

impl NFTAttribute {
    fn as_primitive(&self) -> primitives::NFTAttribute {
        primitives::NFTAttribute {
            name: self.attribute_name.clone(),
            value: self.attribute_value.clone(),
        }
    }
}

// Solana

impl NFTSolanaResult {
    pub fn as_primitive(&self, contract_address: &str) -> Option<primitives::NFTCollection> {
        let chain = Chain::Solana;
        let name = self.collection.clone()?;
        let image_url = self.logo_url.clone()?;
        let description = self.description.clone()?;

        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, contract_address),
            name,
            description: Some(description),
            chain,
            contract_address: contract_address.to_string(),
            image: NFTImage {
                image_url: image_url.clone(),
                preview_image_url: image_url.clone(),
                original_source_url: image_url.clone(),
            },
            is_verified: true,
        })
    }
}

impl NFTSolanaAsset {
    pub fn get_attributes(&self) -> Vec<primitives::NFTAttribute> {
        self.attributes
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|attr| attr.as_primitive())
            .collect()
    }

    pub fn get_image(&self) -> NFTImage {
        let image_url = self.image_uri.clone().unwrap_or_default();
        NFTImage {
            image_url: image_url.clone(),
            preview_image_url: image_url.clone(),
            original_source_url: image_url.clone(),
        }
    }

    pub fn as_primitive(&self, collection_id: &str) -> Option<primitives::NFTAsset> {
        let chain = Chain::Solana;
        let interact_program = self.interact_program.clone()?;

        if interact_program != TOKEN_PROGRAM || self.cnft {
            return None;
        }

        let token_id = self.token_address.clone()?;
        let name = self.name.clone()?;

        Some(primitives::NFTAsset {
            id: primitives::NFTAsset::id(collection_id, token_id.as_str()),
            collection_id: collection_id.to_string(),
            token_id,
            name,
            description: self.description.clone(),
            chain,
            image: self.get_image(),
            token_type: primitives::NFTType::SPL,
            attributes: self.get_attributes(),
        })
    }
}
