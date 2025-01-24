use std::collections::HashMap;

use primitives::{NFTAsset, NFTCollection, NFTData, NFTImage};

use super::model::{MARKET_MAGICEDEN_ID, MARKET_OPENSEA_ID};

impl super::model::NftResponse {
    pub fn as_primitives(&self) -> Vec<NFTData> {
        let mut result = HashMap::new();
        for nft in &self.nfts {
            if let Some(collection) = nft.as_primitive_collection() {
                result.entry(collection).or_insert_with(Vec::new).push(nft.as_primitive_asset());
            }
        }
        result
            .into_iter()
            .map(|(collection, assets)| {
                let collection = collection.clone();
                let assets = assets.into_iter().flatten().collect();
                NFTData { collection, assets }
            })
            .collect::<Vec<_>>()
    }
}

impl super::model::Nft {
    pub fn as_chain(&self) -> Option<primitives::Chain> {
        match self.chain.as_str() {
            "ethereum" => Some(primitives::Chain::Ethereum),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<primitives::NFTType> {
        match self.contract.r#type.as_str() {
            "ERC721" => Some(primitives::NFTType::ERC721),
            "ERC1155" => Some(primitives::NFTType::ERC1155),
            _ => None,
        }
    }

    pub fn is_verified(&self) -> bool {
        self.collection
            .marketplace_pages
            .iter()
            .any(|page| [MARKET_OPENSEA_ID, MARKET_MAGICEDEN_ID].contains(&page.marketplace_id.as_str()) && page.verified.unwrap_or_default())
    }

    pub fn as_primitive_collection(&self) -> Option<primitives::NFTCollection> {
        let chain = self.as_chain()?;
        let id = NFTCollection::id(chain, &self.contract_address);
        Some(primitives::NFTCollection {
            id,
            name: self.collection.name.clone(),
            description: self.collection.description.clone(),
            chain,
            contract_address: self.contract_address.clone(),
            image: NFTImage {
                image_url: self.collection.image_url.clone(),
                preview_image_url: self.collection.image_url.clone(),
                original_source_url: self.collection.image_url.clone(),
            },
            is_verified: self.is_verified(),
        })
    }

    pub fn as_primitive_asset(&self) -> Option<primitives::NFTAsset> {
        let chain: primitives::Chain = self.as_chain()?;
        let collection_id = NFTCollection::id(chain, &self.contract_address);
        let id = NFTAsset::id(collection_id.as_str(), &self.token_id);
        Some(primitives::NFTAsset {
            id,
            token_id: self.token_id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            image: NFTImage {
                image_url: self.previews.image_medium_url.clone(),
                preview_image_url: self.previews.image_small_url.clone(),
                original_source_url: self.previews.image_large_url.clone(),
            },
            collection_id,
            token_type: self.as_type()?,
            chain,
            attributes: self.extra_metadata.attributes.iter().map(|attr| attr.as_primitive()).collect(),
        })
    }
}

impl super::model::Attribute {
    pub fn as_primitive(&self) -> primitives::NFTAttribute {
        primitives::NFTAttribute {
            name: self.trait_type.clone(),
            value: self.value.clone(),
        }
    }
}
