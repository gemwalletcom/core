use std::{collections::HashSet, error::Error, sync::Arc, vec};

use nft_provider::{NFTProviderClient, NFTProviderConfig};
use primitives::{Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};
use std::collections::HashMap;
use storage::Database;

use crate::image_fetcher::ImageFetcher;

#[derive(Clone)]
pub struct NFTClient {
    database: Database,
    nft: NFTProviderClient,
    image_fetcher: Arc<ImageFetcher>,
}

impl NFTClient {
    pub fn new(database: Database, config: NFTProviderConfig) -> Self {
        Self {
            database,
            nft: NFTProviderClient::new(config),
            image_fetcher: Arc::new(ImageFetcher::new()),
        }
    }

    pub async fn update_collection(&self, _collection_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn update_asset(&self, _asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn get_nft_assets(&self, device_id: &str, wallet_index: i32) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.get_subscriptions(device_id, wallet_index)?;
        let addresses: HashMap<Chain, String> = subscriptions.into_iter().map(|x| (x.chain, x.address)).collect();
        self.fetch_assets_for_addresses(addresses).await
    }

    pub async fn preload(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let collection_ids: HashSet<NFTCollectionId> = assets.clone().into_iter().map(|x| x.get_collection_id()).collect();
        self.preload_collections(collection_ids.into_iter().collect()).await?;
        self.preload_assets(assets.clone()).await?;
        self.get_nfts(assets).await
    }

    pub async fn preload_collections(&self, collection_ids: Vec<NFTCollectionId>) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let ids = collection_ids.iter().map(|x| x.id()).collect();
        let existing_collection_ids: Vec<String> = self
            .database
            .client()?
            .nft()
            .get_nft_collections(ids)?
            .into_iter()
            .map(|collection| collection.id)
            .collect();
        let missing_collection_ids = collection_ids
            .into_iter()
            .filter(|id| !existing_collection_ids.contains(&id.id()))
            .collect::<Vec<_>>();

        let mut collections = Vec::new();
        for collection_id in missing_collection_ids {
            match self.nft.get_collection(collection_id.clone()).await {
                Ok(collection) => collections.push(collection),
                Err(e) => println!("nft preload collection {} error: {e}", collection_id.id()),
            }
        }
        let new_collections = collections.clone().into_iter().map(storage::models::NftCollection::from_primitive).collect();

        let links: Vec<storage::models::NftLink> = collections
            .clone()
            .into_iter()
            .flat_map(|x| {
                x.clone()
                    .links
                    .into_iter()
                    .map(move |link| storage::models::NftLink::from_primitive(&x.id.clone(), link))
            })
            .filter(|x| !x.url.is_empty())
            .collect();

        self.database.client()?.nft().add_nft_collections(new_collections)?;
        self.database.client()?.nft().add_nft_collections_links(links)?;

        Ok(collections)
    }

    pub async fn preload_assets(&self, asset_ids: Vec<NFTAssetId>) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let ids = asset_ids.iter().map(|x| x.to_string()).collect();
        let existing_asset_ids: Vec<String> = self.database.client()?.nft().get_nft_assets(ids)?.into_iter().map(|x| x.id).collect();
        let missing_asset_ids = asset_ids
            .into_iter()
            .filter(|id| !existing_asset_ids.contains(&id.to_string()))
            .collect::<Vec<_>>();

        let mut assets = Vec::new();
        for asset_id in missing_asset_ids {
            match self.nft.get_asset(asset_id).await {
                Ok(asset) => assets.push(asset),
                Err(e) => println!("nft preload asset error: {e}"),
            }
        }
        let new_assets = assets
            .clone()
            .into_iter()
            .clone()
            .map(storage::models::NftAsset::from_primitive)
            .collect::<Vec<_>>();

        self.database.client()?.nft().add_nft_assets(new_assets)?;

        Ok(assets)
    }

    pub fn get_subscriptions(&self, device_id: &str, wallet_index: i32) -> Result<Vec<primitives::Subscription>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.subscriptions().get_subscriptions_by_device_id(device_id, Some(wallet_index))?)
    }

    pub async fn get_nft_assets_by_chain(&self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let addresses = [(chain, address.to_string())];
        let assets = self.nft.get_assets(addresses.into()).await?;
        self.preload(assets).await
    }

    pub fn get_nft_collection_data(&self, id: &str) -> Result<NFTData, Box<dyn Error + Send + Sync>> {
        let collection = self.get_collection(id)?;
        Ok(NFTData { collection, assets: vec![] })
    }

    pub fn get_collection(&self, collection_id: &str) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.database.client()?.nft().get_nft_collection(collection_id)?;
        let links: Vec<primitives::AssetLink> = self
            .database
            .client()?
            .nft()
            .get_nft_collection_links(collection_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();

        Ok(collection.as_primitive(links))
    }

    pub fn get_assets(&self, asset_ids: Vec<String>) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.nft().get_nft_assets(asset_ids)?.into_iter().map(|x| x.as_primitive()).collect())
    }

    pub fn get_nft_asset(&self, id: &str) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.nft().get_nft_asset(id)?.as_primitive())
    }

    // computed nfts from db
    async fn get_nfts(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        // cache collections and assets locally

        let collectibles: HashMap<NFTCollectionId, Vec<NFTAssetId>> = assets.clone().into_iter().fold(HashMap::new(), |mut acc, asset| {
            acc.entry(asset.get_collection_id()).or_default().push(asset);
            acc
        });

        collectibles
            .into_iter()
            .map(|x| -> Result<NFTData, Box<dyn Error + Send + Sync>> {
                let collection_id = x.0.id();
                let asset_ids = x.1.into_iter().map(|x| x.to_string()).collect();
                let collection = self.get_collection(collection_id.as_str())?;
                let assets = self.get_assets(asset_ids)?;
                Ok(NFTData { collection, assets })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_nft_asset_image(&self, asset_id: &str) -> Result<(Vec<u8>, Option<String>, HashMap<String, String>), Box<dyn Error + Send + Sync>> {
        let asset = self.get_nft_asset(asset_id)?;
        self.image_fetcher.fetch(&asset.images.preview.url).await
    }

    pub async fn fetch_assets_for_addresses(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let asset_ids = self.nft.get_assets(addresses).await?;
        self.preload(asset_ids.clone()).await
    }
}
