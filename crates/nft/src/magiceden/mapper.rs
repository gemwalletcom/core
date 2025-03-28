use primitives::{Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImage, NFTImageOld, NFTImages, NFTType};

use super::model::{Collection, Nft, Trait};

impl Nft {
    pub fn as_asset_id(&self, chain: Chain) -> Option<NFTAssetId> {
        Some(NFTAssetId::new(chain, &self.collection, &self.mint_address))
    }

    pub fn as_primitive(&self, asset: NFTAssetId, owner: String) -> Option<NFTAsset> {
        let traits = self.attributes.clone();
        Some(NFTAsset {
            id: asset.to_string(),
            collection_id: asset.get_collection_id().id(),
            contract_address: Some(owner),
            token_id: asset.token_id,
            token_type: NFTType::SPL,
            name: self.name.clone(),
            description: None,
            chain: asset.chain,
            image: NFTImageOld {
                image_url: self.image.clone(),
                preview_image_url: self.image.clone(),
                original_source_url: self.image.clone(),
            },
            images: NFTImages {
                preview: NFTImage::from_url(&self.image),
                original: NFTImage::from_url(&self.image),
            },
            attributes: traits.iter().flat_map(|x| x.as_attribute()).collect(),
        })
    }
}

impl Trait {
    pub fn as_attribute(&self) -> Option<NFTAttribute> {
        let value = self.value.as_str()?.to_string();
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
            symbol: self.symbol.clone(),
            description: Some(self.description.clone()),
            chain: collection.chain,
            contract_address: self.on_chain_collection_address.clone(),
            image: NFTImageOld {
                image_url: self.image.clone(),
                preview_image_url: self.image.clone(),
                original_source_url: self.image.clone(),
            },
            images: NFTImages {
                preview: NFTImage::from_url(&self.image),
                original: NFTImage::from_url(&self.image),
            },
            links: self.as_links(),
            is_verified: true,
        }
    }

    pub fn as_links(&self) -> Vec<primitives::AssetLink> {
        let mut links = vec![];
        if let Some(x) = self.twitter.clone() {
            links.push(primitives::AssetLink::new(x.as_str(), LinkType::X));
        }
        if let Some(url) = self.website.clone() {
            links.push(primitives::AssetLink::new(url.as_str(), LinkType::Website));
        }
        if let Some(discord) = self.discord.clone() {
            links.push(primitives::AssetLink::new(discord.as_str(), LinkType::Discord));
        }

        links
    }
}
