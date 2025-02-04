use primitives::{Chain, NFTAssetId, NFTImage};
use serde::{Deserialize, Serialize};

use super::model::NFTAttribute;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTTonResult {
    pub contract_name: Option<String>,
    pub contract_address: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub assets: Vec<NFTTonAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTTonAsset {
    pub token_address: Option<String>,
    pub token_id: Option<String>,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<NFTAttribute>>,
}

impl NFTTonResult {
    pub fn as_primitive(&self) -> Option<primitives::NFTCollection> {
        let chain = Chain::Ton;
        let contract_address = self.contract_address.clone()?;
        let name = self.contract_name.clone()?;
        let image_url = self.logo_url.clone()?;
        let description = self.description.clone();

        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, contract_address.as_str()),
            name,
            description,
            chain,
            contract_address: contract_address.to_string(),
            image: NFTImage {
                image_url: image_url.clone(),
                preview_image_url: image_url.clone(),
                original_source_url: image_url.clone(),
            },
            is_verified: false,
            links: vec![],
        })
    }
}

impl NFTTonAsset {
    pub fn get_image(&self) -> NFTImage {
        let image_url = self.image_uri.clone().unwrap_or_default();
        NFTImage {
            image_url: image_url.clone(),
            preview_image_url: image_url.clone(),
            original_source_url: image_url.clone(),
        }
    }

    pub fn as_primitive(&self, contract_address: &str) -> Option<primitives::NFTAsset> {
        let chain = Chain::Ton;
        let token_id = self.token_address.clone()?;
        let name = self.name.clone()?;
        let id = NFTAssetId::new(chain, contract_address, token_id.as_str());

        Some(primitives::NFTAsset {
            id: id.to_string(),
            collection_id: primitives::NFTCollection::id(chain, contract_address),
            contract_address: Some(contract_address.to_string()),
            token_id,
            name,
            description: self.description.clone(),
            chain: Chain::Ton,
            image: self.get_image(),
            token_type: primitives::NFTType::JETTON,
            attributes: vec![], //self.get_attributes(), //TODO: Reuse
        })
    }
}
