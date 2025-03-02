use primitives::{Chain, NFTAssetId, NFTImage};
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

impl NFTSolanaAsset {
    pub fn as_asset_id(&self) -> Option<NFTAssetId> {
        let token_id = self.token_address.clone().unwrap_or_default();
        let contract_address = self.get_contract_address().unwrap_or_default();
        Some(NFTAssetId::new(Chain::Solana, &contract_address, token_id.as_str()))
    }
}

impl NFTSolanaAsset {
    pub fn get_contract_address(&self) -> Option<String> {
        self.interact_program.clone()
    }
}

impl NFTSolanaResult {
    pub fn as_primitive_asset_ids(&self) -> Vec<NFTAssetId> {
        self.assets.iter().flat_map(|x| x.as_asset_id()).collect()
    }
    pub fn as_primitive(&self, asset: NFTSolanaAsset) -> Option<primitives::NFTCollection> {
        let chain = Chain::Solana;
        let name = self.get_name()?;
        let image_url = self.logo_url.clone()?;
        let description = self.description.clone();
        let contract_address = asset.get_contract_address()?;

        Some(primitives::NFTCollection {
            id: primitives::NFTCollection::id(chain, &contract_address),
            name,
            symbol: None,
            description,
            chain,
            contract_address: contract_address.to_string(),
            image: NFTImage {
                image_url: image_url.clone(),
                preview_image_url: image_url.clone(),
                original_source_url: image_url.clone(),
            },
            links: vec![],
            is_verified: true,
        })
    }
}

impl NFTSolanaAsset {
    pub fn get_attributes(&self) -> Vec<primitives::NFTAttribute> {
        self.attributes.clone().unwrap_or_default().into_iter().flat_map(|x| x.as_primitive()).collect()
    }

    pub fn get_image(&self) -> NFTImage {
        let image_url = self.image_uri.clone().unwrap_or_default();
        NFTImage {
            image_url: image_url.clone(),
            preview_image_url: image_url.clone(),
            original_source_url: image_url.clone(),
        }
    }

    pub fn as_primitive(&self) -> Option<primitives::NFTAsset> {
        let token_id = self.token_address.clone()?;
        let name = self.name.clone()?;
        let contract_address = self.get_contract_address()?;
        let id = NFTAssetId::new(Chain::Solana, &contract_address, token_id.as_str());

        Some(primitives::NFTAsset {
            id: id.to_string(),
            contract_address: Some(contract_address.to_string()),
            collection_id: id.get_collection_id().id(),
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
