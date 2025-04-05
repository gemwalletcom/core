use gem_evm::ethereum_address_checksum;
use primitives::{Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImageOld, NFTImages, NFTResource, NFTType};

use super::model::{Collection, Nft, NftAsset, Trait};

impl Nft {
    pub fn as_primitive(&self, asset: NFTAssetId) -> Option<NFTAsset> {
        let traits = self.traits.clone().unwrap_or_default();
        Some(NFTAsset {
            id: asset.to_string(),
            collection_id: asset.get_collection_id().id(),
            contract_address: Some(asset.contract_address),
            token_id: asset.token_id,
            token_type: self.as_type()?,
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: asset.chain,
            image: NFTImageOld {
                image_url: self.display_image_url.clone(),
                preview_image_url: self.display_image_url.clone(),
                original_source_url: self.display_image_url.clone(),
            },
            resource: NFTResource::from_url(&self.image_url),
            images: NFTImages {
                preview: NFTResource::from_url(&self.display_image_url),
            },
            attributes: traits.iter().flat_map(|x| x.as_attribute()).collect(),
        })
    }

    fn as_type(&self) -> Option<NFTType> {
        match self.token_standard.as_str() {
            "erc1155" => Some(NFTType::ERC1155),
            "erc721" => Some(NFTType::ERC721),
            _ => None,
        }
    }
}

impl NftAsset {
    pub fn as_asset_id(&self, chain: Chain) -> Option<NFTAssetId> {
        let contract_address = ethereum_address_checksum(&self.contract).ok()?;
        Some(NFTAssetId::new(chain, &contract_address, &self.identifier))
    }
}

impl Trait {
    pub fn as_attribute(&self) -> Option<NFTAttribute> {
        let value = self.value.as_str()?.to_string();
        if value == "None" {
            return None;
        }
        Some(NFTAttribute {
            name: self.trait_type.clone(),
            value,
            percentage: None,
        })
    }
}

impl Collection {
    pub fn as_primitive(&self, collection: NFTCollectionId) -> primitives::NFTCollection {
        primitives::NFTCollection {
            id: collection.id(),
            name: self.name.clone(),
            symbol: Some(self.collection.clone()),
            description: Some(self.description.clone()),
            chain: collection.chain,
            contract_address: collection.contract_address.clone(),
            image: NFTImageOld {
                image_url: self.image_url.clone(),
                preview_image_url: self.image_url.clone(),
                original_source_url: self.image_url.clone(),
            },
            images: NFTImages {
                preview: NFTResource::from_url(&self.image_url),
            },
            links: self.as_links(),
            is_verified: true,
        }
    }

    pub fn as_links(&self) -> Vec<primitives::AssetLink> {
        vec![
            primitives::AssetLink::new(self.opensea_url.as_str(), LinkType::OpenSea),
            primitives::AssetLink::new(self.project_url.as_str(), LinkType::Website),
            primitives::AssetLink::new(self.discord_url.as_str(), LinkType::Discord),
            primitives::AssetLink::new(self.telegram_url.as_str(), LinkType::Telegram),
        ]
    }
}
