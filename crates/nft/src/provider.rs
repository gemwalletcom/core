use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

#[async_trait]
pub trait NFTProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chains(&self) -> &'static [NFTChain];
    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>>;
    async fn get_collection(&self, collection: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>>;
    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>>;
}

pub struct NFTProviders {
    providers: Vec<Arc<dyn NFTProvider>>,
}

impl NFTProviders {
    pub fn new(providers: Vec<Arc<dyn NFTProvider>>) -> Self {
        Self { providers }
    }

    fn providers_for_chain(&self, chain: Chain) -> impl Iterator<Item = &Arc<dyn NFTProvider>> {
        self.providers
            .iter()
            .filter(move |provider| provider.chains().iter().any(|nft_chain| Chain::from(*nft_chain) == chain))
    }

    async fn fetch_assets(chain: Chain, address: String, providers: impl Iterator<Item = &Arc<dyn NFTProvider>>) -> Vec<NFTAssetId> {
        for provider in providers {
            if let Ok(ids) = provider.get_assets(chain, address.clone()).await {
                return ids;
            }
        }
        vec![]
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Vec<NFTAssetId> {
        let futures = addresses.into_iter().map(|(chain, address)| {
            let providers = self.providers_for_chain(chain);
            async move { Self::fetch_assets(chain, address, providers).await }
        });

        futures::future::join_all(futures).await.into_iter().flatten().collect()
    }

    pub async fn get_asset_ids(&self, chain: Chain, address: &str) -> Vec<NFTAssetId> {
        let providers = self.providers_for_chain(chain);
        Self::fetch_assets(chain, address.to_string(), providers).await
    }

    pub async fn get_collection(&self, collection_id: NFTCollectionId) -> Option<NFTCollection> {
        for provider in self.providers_for_chain(collection_id.chain) {
            if let Ok(collection) = provider.get_collection(collection_id.clone()).await {
                return Some(collection);
            }
        }
        None
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Option<NFTAsset> {
        for provider in self.providers_for_chain(asset_id.chain) {
            if let Ok(asset) = provider.get_asset(asset_id.clone()).await {
                return Some(asset);
            }
        }
        None
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
