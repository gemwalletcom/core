use gem_evm::ethereum_address_checksum;
use primitives::{AssetLink, Chain, LinkType, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};

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
        let resource_url = self.resource_url();
        let preview_url = self.preview_url();

        Some(NFTAsset {
            id: asset.to_string(),
            collection_id: asset.get_collection_id().id(),
            contract_address: Some(asset.contract_address),
            token_id: asset.token_id,
            token_type: self.as_type()?,
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: asset.chain,
            resource: NFTResource::from_url(resource_url),
            images: NFTImages {
                preview: NFTResource::from_url(preview_url),
            },
            attributes: traits.iter().flat_map(|x| x.as_attribute()).collect(),
        })
    }

    fn resource_url(&self) -> &str {
        self.image_url
            .as_deref()
            .or(self.original_image_url.as_deref())
            .or(self.display_image_url.as_deref())
            .unwrap_or_default()
    }

    fn preview_url(&self) -> &str {
        self.display_image_url
            .as_deref()
            .or(self.image_url.as_deref())
            .or(self.original_image_url.as_deref())
            .unwrap_or_default()
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
        let is_verified = self.safelist_status.as_deref() == Some("verified");

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
            status: VerificationStatus::from_verified(is_verified),
            links: self.as_links(),
            is_verified,
        }
    }

    pub fn as_links(&self) -> Vec<AssetLink> {
        [
            self.opensea_url.as_deref().map(|u| AssetLink::new(u, LinkType::OpenSea)),
            self.project_url.as_deref().map(|u| AssetLink::new(u, LinkType::Website)),
            self.discord_url.as_deref().map(|u| AssetLink::new(u, LinkType::Discord)),
            self.telegram_url.as_deref().map(|u| AssetLink::new(u, LinkType::Telegram)),
        ]
        .into_iter()
        .flatten()
        .filter(|link| !link.url.is_empty())
        .collect()
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

        let nft_asset = response.nft.as_primitive(asset_id).unwrap();

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

    #[test]
    fn test_map_asset_with_null_image_urls() {
        let response: NftResponse = serde_json::from_str(include_str!("../../../testdata/opensea/asset_null_images.json")).unwrap();
        let asset_id = NFTAssetId::new(
            Chain::Ethereum,
            "0xd4416b13d2b3a9abae7acd5d6c2bbdbe25686401",
            "66972740172774133895361774757009899712806299063970949277266423600598010529206",
        );

        let nft_asset = map_asset(response, asset_id).unwrap();

        assert_eq!(nft_asset.chain, Chain::Ethereum);
        assert_eq!(nft_asset.token_id, "66972740172774133895361774757009899712806299063970949277266423600598010529206");
        assert_eq!(nft_asset.name, "gemdev.eth");
        assert_eq!(
            nft_asset.resource.url,
            "https://metadata.ens.domains/mainnet/0xd4416b13d2b3a9abae7acd5d6c2bbdbe25686401/0x94113a45c5bedf735911bf707d70a6c05d9d99e76ece7e904c0eeda6591785b6/image"
        );
        assert_eq!(nft_asset.images.preview.url, nft_asset.resource.url);
    }
}
