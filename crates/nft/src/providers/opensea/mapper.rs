use gem_evm::ethereum_address_checksum;
use primitives::{AssetLink, Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImages, NFTResource, NFTType};

use super::model::{Collection, Nft, NftAsset, NftResponse, NftsResponse, Trait};

pub fn map_assets(response: NftsResponse, chain: Chain) -> Vec<NFTAssetId> {
    response.nfts.into_iter().flat_map(|x| x.as_asset_id(chain)).collect()
}

pub fn map_collection(collection: Collection, collection_id: NFTCollectionId) -> primitives::NFTCollection {
    collection.as_primitive(collection_id)
}

pub fn map_asset(response: NftResponse, asset_id: NFTAssetId) -> Option<NFTAsset> {
    response.nft.as_primitive(asset_id)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::opensea::model::{Collection, NftResponse, NftsResponse};
    use crate::testkit::TEST_ETHEREUM_CONTRACT_ADDRESS;
    use primitives::{Chain, NFTCollectionId};

    #[test]
    fn test_map_assets() {
        let response: NftsResponse = serde_json::from_str(include_str!("../../../testdata/opensea/assets.json")).unwrap();
        let asset_ids = map_assets(response, Chain::Ethereum);

        assert!(!asset_ids.is_empty());
        if let Some(first_asset) = asset_ids.first() {
            assert_eq!(first_asset.chain, Chain::Ethereum);
            assert!(!first_asset.contract_address.is_empty());
            assert!(!first_asset.token_id.is_empty());
            assert!(first_asset.contract_address.starts_with("0x"));
        }
    }

    #[test]
    fn test_map_collection() {
        let collection: Collection = serde_json::from_str(include_str!("../../../testdata/opensea/collection.json")).unwrap();
        let collection_id = NFTCollectionId::new(Chain::Ethereum, TEST_ETHEREUM_CONTRACT_ADDRESS);
        let nft_collection = map_collection(collection, collection_id);

        assert_eq!(nft_collection.chain, Chain::Ethereum);
        assert_eq!(nft_collection.name, "Bored Ape Yacht Club");
        assert!(nft_collection.description.is_some());
        assert!(nft_collection.description.as_ref().unwrap().contains("10,000 unique Bored Ape NFTs"));
        assert!(!nft_collection.links.is_empty());
        assert!(nft_collection.links.iter().any(|link| link.url.contains("opensea.io")));
    }

    #[test]
    fn test_map_asset() {
        let response: NftResponse = serde_json::from_str(include_str!("../../../testdata/opensea/asset.json")).unwrap();
        let asset_id = NFTAssetId::new(Chain::Ethereum, TEST_ETHEREUM_CONTRACT_ADDRESS, "1");
        let nft_asset = map_asset(response, asset_id).expect("Failed to map asset");

        assert_eq!(nft_asset.chain, Chain::Ethereum);
        assert_eq!(nft_asset.token_id, "1");
        assert_eq!(nft_asset.name, "#1");
        assert!(nft_asset.contract_address.is_some());
        assert!(!nft_asset.attributes.is_empty());

        let mouth_trait = nft_asset.attributes.iter().find(|attr| attr.name == "Mouth");
        assert!(mouth_trait.is_some());
        assert_eq!(mouth_trait.unwrap().value, "Grin");
    }

    #[test]
    fn test_asset_id_mapping() {
        let response: NftsResponse = serde_json::from_str(include_str!("../../../testdata/opensea/assets.json")).unwrap();

        let chain = Chain::Ethereum;
        let asset_ids: Vec<NFTAssetId> = response.nfts.into_iter().flat_map(|nft_asset| nft_asset.as_asset_id(chain)).collect();

        assert!(!asset_ids.is_empty());

        if let Some(first_asset) = asset_ids.first() {
            assert_eq!(first_asset.chain, Chain::Ethereum);
            assert!(!first_asset.contract_address.is_empty());
            assert!(!first_asset.token_id.is_empty());
            assert!(first_asset.contract_address.starts_with("0x"));
        }
    }

    #[test]
    fn test_asset_primitive_mapping() {
        let response: NftResponse = serde_json::from_str(include_str!("../../../testdata/opensea/asset.json")).unwrap();

        let asset_id = NFTAssetId::new(Chain::Ethereum, "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D", "1");

        let nft_asset = response.nft.as_primitive(asset_id).expect("Failed to map asset");

        assert_eq!(nft_asset.chain, Chain::Ethereum);
        assert_eq!(nft_asset.token_id, "1");
        assert_eq!(nft_asset.name, "#1");
        assert!(nft_asset.contract_address.is_some());
        assert!(!nft_asset.attributes.is_empty());

        let mouth_trait = nft_asset.attributes.iter().find(|attr| attr.name == "Mouth");
        assert!(mouth_trait.is_some());
        assert_eq!(mouth_trait.unwrap().value, "Grin");
    }

    #[test]
    fn test_collection_primitive_mapping() {
        let collection: Collection = serde_json::from_str(include_str!("../../../testdata/opensea/collection.json")).unwrap();

        let collection_id = NFTCollectionId::new(Chain::Ethereum, "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D");
        let nft_collection = collection.as_primitive(collection_id);

        assert_eq!(nft_collection.chain, Chain::Ethereum);
        assert_eq!(nft_collection.name, "Bored Ape Yacht Club");
        assert!(nft_collection.description.is_some());
        assert!(nft_collection.description.as_ref().unwrap().contains("10,000 unique Bored Ape NFTs"));

        assert!(!nft_collection.links.is_empty());

        assert!(nft_collection.links.iter().any(|link| link.url.contains("opensea.io")));
        assert!(nft_collection.links.iter().any(|link| link.url.contains("boredapeyachtclub.com")));
        assert!(nft_collection.links.iter().any(|link| link.url.contains("discord.gg")));
    }
}
