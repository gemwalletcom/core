use gem_evm::ethereum_address_checksum;
use primitives::{AssetLink, Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImages, NFTResource, NFTType};

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
            description: self.description.clone(),
            chain: collection.chain,
            contract_address: collection.contract_address.clone(),
            images: NFTImages {
                preview: NFTResource::from_url(self.image_url.as_deref().unwrap_or("")),
            },
            links: self.as_links(),
            is_verified: true,
        }
    }

    pub fn as_links(&self) -> Vec<AssetLink> {
        let mut links = Vec::new();
        
        if let Some(opensea_url) = &self.opensea_url {
            links.push(AssetLink::new(opensea_url.as_str(), LinkType::OpenSea));
        }
        if let Some(project_url) = &self.project_url {
            links.push(AssetLink::new(project_url.as_str(), LinkType::Website));
        }
        if let Some(discord_url) = &self.discord_url {
            links.push(AssetLink::new(discord_url.as_str(), LinkType::Discord));
        }
        if let Some(telegram_url) = &self.telegram_url {
            links.push(AssetLink::new(telegram_url.as_str(), LinkType::Telegram));
        }
        
        links
    }
}
