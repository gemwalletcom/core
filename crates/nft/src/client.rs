use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use primitives::{AssetId, Chain, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};
use storage::database::devices::DevicesStore;
use storage::{Database, NftRepository, WalletsRepository};

use crate::NFTProviderConfig;
use crate::provider_client::NFTProviderClient;

pub struct NFTClient {
    database: Database,
    provider_client: Arc<NFTProviderClient>,
    assets_url: String,
}

impl NFTClient {
    pub fn new(database: Database, provider_client: Arc<NFTProviderClient>, assets_url: String) -> Self {
        Self {
            database,
            provider_client,
            assets_url,
        }
    }

    pub fn from_config(database: Database, config: NFTProviderConfig, assets_url: String) -> Self {
        Self::new(database, Arc::new(NFTProviderClient::new(config)), assets_url)
    }

    pub async fn update_collection(&self, _collection_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn update_asset(&self, _asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn get_nft_assets_by_wallet_id(&self, device_id: i32, wallet_id: i32) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
        let addresses: HashMap<Chain, String> = subscriptions.into_iter().map(|(sub, addr)| (sub.chain.0, addr.address)).collect();
        let mut result = Vec::new();
        for (chain, address) in addresses {
            if let Ok(data) = self.provider_client.get_nft_data(chain, &address).await {
                result.extend(data);
            }
        }
        Ok(result.into_iter().map(|d| self.with_urls_data(d)).collect())
    }

    pub async fn get_nft_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        Ok(self.with_urls_asset(self.provider_client.get_nft_asset(asset_id).await?))
    }

    pub async fn get_nft_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        Ok(self.with_urls_collection(self.provider_client.get_nft_collection(collection_id).await?))
    }

    pub async fn get_nft_data(&self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .provider_client
            .get_nft_data(chain, address)
            .await?
            .into_iter()
            .map(|d| self.with_urls_data(d))
            .collect())
    }

    fn with_urls_asset(&self, asset: NFTAsset) -> NFTAsset {
        let preview_url = format!("{}/nft/asset/{}/preview", self.assets_url, asset.id);
        let resource_url = format!("{}/nft/asset/{}/resource", self.assets_url, asset.id);
        asset.with_urls(preview_url, resource_url)
    }

    fn with_urls_collection(&self, collection: NFTCollection) -> NFTCollection {
        let preview_url = format!("{}/nft/collection/{}/preview", self.assets_url, collection.id);
        collection.with_preview_url(preview_url)
    }

    fn with_urls_data(&self, data: NFTData) -> NFTData {
        NFTData {
            collection: self.with_urls_collection(data.collection),
            assets: data.assets.into_iter().map(|a| self.with_urls_asset(a)).collect(),
        }
    }

    pub async fn preload(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let collection_ids: HashSet<NFTCollectionId> = assets.clone().into_iter().map(|x| x.get_collection_id()).collect();
        self.preload_collections(collection_ids.into_iter().collect()).await?;
        self.preload_assets(assets.clone()).await?;
        self.get_nfts(assets).await
    }

    pub async fn preload_collections(&self, collection_ids: Vec<NFTCollectionId>) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let ids = collection_ids.iter().map(|x| x.id()).collect();
        let existing_collection_ids: Vec<String> = self.database.nft()?.get_nft_collections(ids)?.into_iter().map(|collection| collection.id).collect();
        let missing_collection_ids = collection_ids.into_iter().filter(|id| !existing_collection_ids.contains(&id.id())).collect::<Vec<_>>();

        let mut collections = Vec::new();
        for collection_id in missing_collection_ids {
            if let Ok(collection) = self.provider_client.get_nft_collection(collection_id.clone()).await {
                collections.push(collection);
            }
        }
        let new_collections = collections.clone().into_iter().map(storage::models::NewNftCollectionRow::from_primitive).collect();

        let links: Vec<storage::models::NftLinkRow> = collections
            .clone()
            .into_iter()
            .flat_map(|x| {
                x.clone()
                    .links
                    .into_iter()
                    .map(move |link| storage::models::NftLinkRow::from_primitive(&x.id.clone(), link))
            })
            .filter(|x| !x.url.is_empty())
            .collect();

        self.database.nft()?.add_nft_collections(new_collections)?;
        self.database.nft()?.add_nft_collections_links(links)?;

        Ok(collections)
    }

    pub async fn preload_assets(&self, asset_ids: Vec<NFTAssetId>) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let ids = asset_ids.iter().map(|x| x.to_string()).collect();
        let existing_asset_ids: Vec<String> = self.database.nft()?.get_nft_assets(ids)?.into_iter().map(|x| x.id).collect();
        let missing_asset_ids = asset_ids.into_iter().filter(|id| !existing_asset_ids.contains(&id.to_string())).collect::<Vec<_>>();

        let mut assets = Vec::new();
        for asset_id in missing_asset_ids {
            if let Ok(asset) = self.provider_client.get_nft_asset(asset_id).await {
                assets.push(asset);
            }
        }
        let new_assets = assets.clone().into_iter().clone().map(storage::models::NftAssetRow::from_primitive).collect::<Vec<_>>();

        self.database.nft()?.add_nft_assets(new_assets)?;

        Ok(assets)
    }

    fn load_nft_assets(&self, asset_ids: Vec<String>) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.nft()?.get_nft_assets(asset_ids)?.into_iter().map(|x| x.as_primitive()).collect())
    }

    async fn get_nfts(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let collectibles: HashMap<NFTCollectionId, Vec<NFTAssetId>> = assets.clone().into_iter().fold(HashMap::new(), |mut acc, asset| {
            acc.entry(asset.get_collection_id()).or_default().push(asset);
            acc
        });

        collectibles
            .into_iter()
            .map(|x| -> Result<NFTData, Box<dyn Error + Send + Sync>> {
                let collection_id = x.0.id();
                let asset_ids = x.1.into_iter().map(|x| x.to_string()).collect();
                let collection = self.load_nft_collection(collection_id.as_str())?;
                let assets = self.load_nft_assets(asset_ids)?;
                Ok(NFTData { collection, assets })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn load_nft_asset(&self, asset_id: &str) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.nft()?.get_nft_asset(asset_id)?.as_primitive())
    }

    pub fn load_nft_collection(&self, collection_id: &str) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let row = self.database.nft()?.get_nft_collection(collection_id)?;
        let links = self
            .database
            .nft()?
            .get_nft_collection_links(collection_id)?
            .into_iter()
            .map(|x| x.as_primitive())
            .collect();
        Ok(row.as_primitive(links))
    }

    pub async fn fetch_assets_for_addresses(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let asset_ids = self.provider_client.get_asset_ids_for_addresses(addresses).await;
        self.preload(asset_ids).await
    }

    pub fn report_nft(&self, device_id: &str, collection_id: String, asset_id: Option<AssetId>, reason: Option<String>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let device = DevicesStore::get_device(&mut client, device_id)?;
        let report = storage::models::NewNftReportRow {
            device_id: device.id,
            collection_id,
            asset_id: asset_id.map(Into::into),
            reason,
        };
        client.nft().add_nft_report(report)?;
        Ok(true)
    }
}
