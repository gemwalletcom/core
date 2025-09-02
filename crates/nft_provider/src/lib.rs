use std::sync::Arc;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use primitives::{nft::NFTCollectionId, Chain, NFTAsset, NFTAssetId, NFTCollection};

pub mod config;
pub mod magiceden;
pub mod opensea;
pub mod testkit;

pub use config::NFTProviderConfig;
pub use magiceden::MagicEdenClient;
pub use opensea::OpenSeaClient;

#[async_trait]
pub trait NFTProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn get_chains(&self) -> Vec<Chain>;
    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>>;
    async fn get_collection(&self, collection: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>>;
    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>>;
}

#[allow(unused)]
pub struct NFTProviderClient {
    providers: Vec<Arc<dyn NFTProvider>>,
}

impl NFTProviderClient {
    pub fn new(config: NFTProviderConfig) -> Self {
        let providers: Vec<Arc<dyn NFTProvider>> = vec![
            Arc::new(OpenSeaClient::new(&config.opensea_key)),
            Arc::new(MagicEdenClient::new(&config.magiceden_key)),
        ];

        Self { providers }
    }

    pub fn get_provider_for_chain(&self, chain: Chain) -> Option<Arc<dyn NFTProvider>> {
        self.providers.iter().find(|provider| provider.get_chains().contains(&chain)).cloned()
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let futures: Vec<_> = addresses
            .into_iter()
            .map(|(chain, address)| {
                let address = address.clone();
                async move { self.get_asset_ids(chain, address.as_str()).await }
            })
            .collect();

        Ok(futures::future::try_join_all(futures).await?.into_iter().flatten().collect::<Vec<NFTAssetId>>())
    }

    pub async fn get_asset_ids(&self, chain: Chain, address: &str) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        if let Some(provider) = self.get_provider_for_chain(chain) {
            provider.get_assets(chain, address.to_string()).await
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        if let Some(provider) = self.get_provider_for_chain(collection_id.chain) {
            provider.get_collection(collection_id).await
        } else {
            Err(format!("No provider available for chain: {:?}", collection_id.chain).into())
        }
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        if let Some(provider) = self.get_provider_for_chain(asset_id.chain) {
            provider.get_asset(asset_id).await
        } else {
            Err(format!("No provider available for chain: {:?}", asset_id.chain).into())
        }
    }
}

pub async fn get_image_mime_type(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let response = reqwest::Client::new().head(url).send().await?;
    if let Some(mime_type) = response.headers().get(reqwest::header::CONTENT_TYPE) {
        Ok(mime_type.to_str()?.to_string())
    } else {
        Err("Failed to determine MIME type".into())
    }
}
