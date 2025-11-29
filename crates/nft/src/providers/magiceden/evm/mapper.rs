use gem_evm::ethereum_address_checksum;
use primitives::{Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImages, NFTResource, NFTType};

use super::model::{Attribute, CollectionDetail, TokenAsset, TokenDetail};

pub fn map_assets(assets: Vec<TokenAsset>, chain: Chain) -> Vec<NFTAssetId> {
    assets.into_iter().flat_map(|token_asset| token_asset.asset.as_asset_id(chain)).collect()
}

pub fn map_collection(collection: CollectionDetail, collection_id: NFTCollectionId) -> primitives::NFTCollection {
    collection.as_primitive(collection_id)
}

pub fn map_asset(token: TokenDetail, asset_id: NFTAssetId) -> Option<NFTAsset> {
    token.as_primitive(asset_id)
}

impl TokenDetail {
    pub fn as_asset_id(&self, chain: Chain) -> Option<NFTAssetId> {
        let contract_address = ethereum_address_checksum(&self.collection_id).ok()?;
        Some(NFTAssetId::new(chain, &contract_address, &self.token_id))
    }

    pub fn as_primitive(&self, asset: NFTAssetId) -> Option<NFTAsset> {
        let image_url = self
            .media_v2
            .as_ref()
            .and_then(|m| m.main.as_ref())
            .and_then(|main| main.uri.clone())
            .unwrap_or_default();

        let token_type = match self.standard.as_deref() {
            Some("ERC721") => NFTType::ERC721,
            Some("ERC1155") => NFTType::ERC1155,
            _ => return None,
        };

        Some(NFTAsset {
            id: asset.to_string(),
            collection_id: asset.get_collection_id().id(),
            contract_address: Some(asset.contract_address),
            token_id: asset.token_id,
            token_type,
            name: self.name.clone().unwrap_or_default(),
            description: self.description.clone(),
            chain: asset.chain,
            resource: NFTResource::from_url(&image_url),
            images: NFTImages {
                preview: NFTResource::from_url(&image_url),
            },
            attributes: self.attributes.clone().unwrap_or_default().iter().flat_map(|x| x.as_attribute()).collect(),
        })
    }
}

impl Attribute {
    pub fn as_attribute(&self) -> Option<NFTAttribute> {
        let value = self.value.as_str()?.to_string();
        Some(NFTAttribute {
            name: self.trait_type.clone(),
            value,
            percentage: None,
        })
    }
}

impl CollectionDetail {
    pub fn as_primitive(&self, collection: NFTCollectionId) -> primitives::NFTCollection {
        let image_url = self.media.as_ref().and_then(|m| m.url.clone()).unwrap_or_default();
        let contract_address = ethereum_address_checksum(&self.id).unwrap_or_else(|_| self.id.clone());

        primitives::NFTCollection {
            id: collection.id(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            description: self.description.clone(),
            chain: collection.chain,
            contract_address,
            images: NFTImages {
                preview: NFTResource::from_url(&image_url),
            },
            links: self.as_links(),
            is_verified: true,
        }
    }

    pub fn as_links(&self) -> Vec<primitives::AssetLink> {
        let mut links = vec![];
        if let Some(social) = &self.social {
            if let Some(twitter) = &social.twitter_url {
                links.push(primitives::AssetLink::new(twitter, LinkType::X));
            }
            if let Some(website) = &social.website_url {
                links.push(primitives::AssetLink::new(website, LinkType::Website));
            }
            if let Some(discord) = &social.discord_url {
                links.push(primitives::AssetLink::new(discord, LinkType::Discord));
            }
        }
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::magiceden::evm::model;
    use primitives::{Chain, NFTCollectionId};

    #[test]
    fn test_map_assets() {
        let response: model::TokensResponse = serde_json::from_str(include_str!("../../../testdata/magiceden/evm_assets.json")).unwrap();
        let asset_ids = map_assets(response.assets, Chain::SmartChain);

        assert!(!asset_ids.is_empty());
        if let Some(first_asset) = asset_ids.first() {
            assert_eq!(first_asset.chain, Chain::SmartChain);
            assert!(!first_asset.contract_address.is_empty());
            assert!(!first_asset.token_id.is_empty());
        }
    }

    #[test]
    fn test_map_collection() {
        let response: model::CollectionsResponse = serde_json::from_str(include_str!("../../../testdata/magiceden/evm_collection.json")).unwrap();
        let collection = response.collections.into_iter().next().unwrap();
        let collection_id = NFTCollectionId::new(Chain::SmartChain, &collection.id);
        let nft_collection = map_collection(collection, collection_id);

        assert_eq!(nft_collection.chain, Chain::SmartChain);
        assert_eq!(nft_collection.name, "Reefers by CoralApp");
        assert!(nft_collection.description.is_some());
        assert!(!nft_collection.links.is_empty());
    }

    #[test]
    fn test_map_asset() {
        let asset_response: model::TokenDetailResponse = serde_json::from_str(include_str!("../../../testdata/magiceden/evm_asset.json")).unwrap();
        let token = asset_response.token;
        let asset_id = NFTAssetId::new(Chain::SmartChain, &token.collection_id, &token.token_id);
        let nft_asset = map_asset(token, asset_id).expect("Failed to map asset");

        assert_eq!(nft_asset.chain, Chain::SmartChain);
        assert_eq!(nft_asset.token_id, "410");
        assert!(nft_asset.name.contains("Reefers"));
        assert!(nft_asset.description.is_some());
        assert!(!nft_asset.attributes.is_empty());
    }
}
