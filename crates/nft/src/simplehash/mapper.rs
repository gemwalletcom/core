use std::collections::HashMap;

use primitives::{LinkType, NFTAsset, NFTCollection, NFTData, NFTImage};

use super::model::{MARKET_MAGICEDEN_ID, MARKET_OPENSEA_ID};

impl super::model::NftResponse {
    pub fn as_primitives(&self) -> Vec<NFTData> {
        let mut result = HashMap::new();
        for nft in &self.nfts {
            if let Some(collection) = nft.as_primitive_collection() {
                if nft.asset_has_image() && nft.collection_has_image() {
                    result.entry(collection).or_insert_with(Vec::new).push(nft.as_primitive_asset());
                }
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
            "solana" => Some(primitives::Chain::Solana),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<primitives::NFTType> {
        match self.contract.contract_type.clone()?.as_str() {
            "ERC721" => Some(primitives::NFTType::ERC721),
            "ERC1155" => Some(primitives::NFTType::ERC1155),
            "NonFungible" | "ProgrammableNonFungible" => Some(primitives::NFTType::SPL),
            _ => None,
        }
    }

    pub fn is_verified_collection(&self) -> bool {
        self.collection
            .marketplace_pages
            .iter()
            .any(|page| [MARKET_OPENSEA_ID, MARKET_MAGICEDEN_ID].contains(&page.marketplace_id.as_str()) && page.verified.unwrap_or_default())
    }

    pub fn collection_has_image(&self) -> bool {
        !self.as_primitive_collection_image().image_url.is_empty()
    }

    pub fn asset_has_image(&self) -> bool {
        !self.as_primitive_asset_image().image_url.is_empty()
    }

    pub fn as_primitive_collection(&self) -> Option<primitives::NFTCollection> {
        let chain = self.as_chain()?;
        let id = NFTCollection::id(chain, &self.get_contract_address()?);
        let links = self.as_links();

        Some(primitives::NFTCollection {
            id,
            name: self.collection.name.clone().unwrap_or_default(),
            description: self.collection.description.clone(),
            chain,
            contract_address: self.contract_address.clone(),
            image: self.as_primitive_collection_image(),
            is_verified: self.is_verified_collection(),
            links,
        })
    }

    pub fn as_links(&self) -> Vec<primitives::AssetLink> {
        self.collection.marketplace_pages.iter().flat_map(|x| x.as_primitive()).collect()
    }

    pub fn as_primitive_collection_image(&self) -> primitives::NFTImage {
        if let Some(image_properties) = &self.collection.image_properties {
            if image_properties.mime_type == Some("image/png".to_string()) {
                let image_url = self.collection.image_url.clone().unwrap_or_default();
                return NFTImage {
                    image_url: image_url.clone(),
                    preview_image_url: image_url.clone(),
                    original_source_url: image_url.clone(),
                };
            } else if let Some(image_original_url) = &self.extra_metadata.image_original_url {
                return NFTImage {
                    image_url: image_original_url.clone(),
                    preview_image_url: image_original_url.clone(),
                    original_source_url: image_original_url.clone(),
                };
            }
        }
        NFTImage {
            image_url: "".to_string(),
            preview_image_url: "".to_string(),
            original_source_url: "".to_string(),
        }
    }

    pub fn get_token_id(&self) -> Option<String> {
        match self.as_chain()? {
            primitives::Chain::Solana => Some(self.contract_address.clone()),
            _ => self.token_id.clone(),
        }
    }

    pub fn get_contract_address(&self) -> Option<String> {
        match self.as_chain()? {
            primitives::Chain::Solana => self
                .collection
                .mpl_core_collection_address
                .clone()
                .or_else(|| self.collection.metaplex_mint.clone())
                .or_else(|| self.extra_metadata.token_program.clone()),
            _ => Some(self.contract_address.clone()),
        }
    }

    pub fn as_primitive_asset(&self) -> Option<primitives::NFTAsset> {
        let chain: primitives::Chain = self.as_chain()?;
        let collection_id = NFTCollection::id(chain, &self.get_contract_address()?);
        let token_id = &self.get_token_id()?;
        let id = NFTAsset::id(chain, &self.get_contract_address()?, token_id);
        Some(primitives::NFTAsset {
            id,
            token_id: token_id.to_string(),
            contract_address: Some(self.contract_address.clone()),
            name: self.name.clone().unwrap_or_default(),
            description: self.description.clone(),
            image: self.as_primitive_asset_image(),
            collection_id,
            token_type: self.as_type()?,
            chain,
            attributes: self.extra_metadata.attributes.iter().map(|attr| attr.as_primitive()).collect(),
        })
    }

    pub fn as_primitive_asset_image(&self) -> primitives::NFTImage {
        NFTImage {
            image_url: self.previews.image_medium_url.clone().unwrap_or_default(),
            preview_image_url: self.previews.image_small_url.clone().unwrap_or_default(),
            original_source_url: self.previews.image_large_url.clone().unwrap_or_default(),
        }
    }
}

impl super::model::Attribute {
    pub fn as_primitive(&self) -> primitives::NFTAttribute {
        primitives::NFTAttribute {
            name: self.trait_type.clone(),
            value: self.value.clone(),
            percentage: self.percentage,
        }
    }
}

impl super::model::MarketplacePage {
    pub fn as_primitive(&self) -> Option<primitives::AssetLink> {
        let link_type = match self.marketplace_id.as_str() {
            MARKET_OPENSEA_ID => LinkType::OpenSea,
            MARKET_MAGICEDEN_ID => LinkType::MagicEden,
            _ => return None,
        };
        let url = self.collection_url.clone()?;

        Some(primitives::AssetLink {
            name: link_type.as_ref().to_string(),
            url,
        })
    }
}
