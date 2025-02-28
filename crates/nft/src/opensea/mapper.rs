use std::str::FromStr;

use gem_evm::address::EthereumAddress;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTAttribute, NFTCollectionId, NFTImage, NFTType};

use super::model::{Collection, Nft, Trait};

impl Nft {
    pub fn as_asset_id(&self, chain: Chain) -> Option<NFTAssetId> {
        let contract_address = EthereumAddress::from_str(&self.contract).ok()?.to_checksum();
        Some(NFTAssetId::new(chain, &contract_address, &self.identifier))
    }

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
            image: NFTImage {
                image_url: self.image_url.clone(),
                preview_image_url: self.image_url.clone(),
                original_source_url: self.image_url.clone(),
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

// impl std::str::FromStr for primitives::NFTType {
//     type Err = ();

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "erc1155" => Ok(primitives::NFTType::ERC1155),
//             "erc721" => Ok(primitives::NFTType::ERC721),
//             _ => Err(()),
//         }
//     }
// }

impl Collection {
    pub fn as_primitive(&self, collection: NFTCollectionId) -> primitives::NFTCollection {
        primitives::NFTCollection {
            id: collection.id(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            chain: collection.chain,
            contract_address: collection.contract_address.clone(),
            image: NFTImage {
                image_url: self.image_url.clone(),
                preview_image_url: self.image_url.clone(),
                original_source_url: self.image_url.clone(),
            },
            links: vec![], // TODO
            is_verified: true,
        }
    }
}
