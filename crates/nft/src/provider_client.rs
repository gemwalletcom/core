use std::collections::HashMap;
use std::error::Error;

use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};

use crate::NFTProviderConfig;
use crate::factory::NFTProviderFactory;
use crate::provider::NFTProviders;

pub struct NFTProviderClient {
    providers: NFTProviders,
}

impl NFTProviderClient {
    pub fn new(config: NFTProviderConfig) -> Self {
        let providers = NFTProviderFactory::new_providers(config);
        Self {
            providers: NFTProviders::new(providers),
        }
    }

    pub async fn get_nft_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        self.providers.get_asset(asset_id).await.ok_or_else(|| "Asset not found".into())
    }

    pub async fn get_nft_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        self.providers.get_collection(collection_id).await.ok_or_else(|| "Collection not found".into())
    }

    pub async fn get_nft_data(&self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        self.providers.get_nft_data(chain, address).await
    }

    pub async fn get_asset_ids_for_addresses(&self, addresses: HashMap<Chain, String>) -> Vec<NFTAssetId> {
        self.providers.get_assets(addresses).await
    }
}
