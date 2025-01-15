use gem_solana::TOKEN_PROGRAM;
use primitives::{Chain, NFTImage};
use serde::{Deserialize, Serialize};

use super::model::NFTAttribute;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTSolanaResult {
    pub contract_name: Option<String>,
    pub contract_address: Option<String>,

    pub collection: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub assets: Vec<NFTSolanaAsset>,
}

impl NFTSolanaResult {
    pub fn get_name(&self) -> Option<String> {
        if let Some(name) = self.contract_name.clone() {
            Some(name)
        } else {
            self.collection.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTSolanaAsset {
    pub token_address: Option<String>,
    pub interact_program: Option<String>,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<NFTAttribute>>,
    pub cnft: Option<bool>,
}

impl NFTSolanaResult {
    pub fn as_primitive(&self, contract_address: &str) -> Option<primitives::NFTCollection> {
        let chain = Chain::Solana;
        let name = self.get_name()?;
        let image_url = self.logo_url.clone()?;
        let description = self.description.clone();

        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, contract_address),
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
        })
    }
}

impl NFTSolanaAsset {
    pub fn get_attributes(&self) -> Vec<primitives::NFTAttribute> {
        self.attributes
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|attr| attr.as_primitive())
            .collect()
    }

    pub fn get_image(&self) -> NFTImage {
        let image_url = self.image_uri.clone().unwrap_or_default();
        NFTImage {
            image_url: image_url.clone(),
            preview_image_url: image_url.clone(),
            original_source_url: image_url.clone(),
        }
    }

    pub fn as_primitive(&self, collection_id: &str) -> Option<primitives::NFTAsset> {
        let interact_program = self.interact_program.clone()?;

        if interact_program != TOKEN_PROGRAM || self.cnft == Some(true) {
            return None;
        }

        let token_id = self.token_address.clone()?;
        let name = self.name.clone()?;

        Some(primitives::NFTAsset {
            id: primitives::NFTAsset::id(collection_id, token_id.as_str()),
            collection_id: collection_id.to_string(),
            token_id,
            name,
            description: self.description.clone(),
            chain: Chain::Solana,
            image: self.get_image(),
            token_type: primitives::NFTType::SPL,
            attributes: self.get_attributes(),
        })
    }
}
