use std::collections::{HashMap, HashSet};
use std::error::Error;

use primitives::{AssetId, Chain, ImageFormatter, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};
use storage::database::devices::DevicesStore;
use storage::database::nft::{NftAssetFilter, NftCollectionFilter};
use storage::models::{NewNftAssetRow, NewNftCollectionRow, NftLinkRow};
use storage::{Database, NftRepository, WalletsRepository};

use crate::NFTProviderConfig;
use crate::provider_client::NFTProviderClient;

pub struct NFTClient {
    database: Database,
    provider_client: NFTProviderClient,
    assets_url: String,
}

impl NFTClient {
    pub fn new(database: Database, provider_client: NFTProviderClient, assets_url: String) -> Self {
        Self {
            database,
            provider_client,
            assets_url,
        }
    }

    pub fn from_config(database: Database, config: NFTProviderConfig, assets_url: String) -> Self {
        Self::new(database, NFTProviderClient::new(config), assets_url)
    }

    pub async fn update_collection(&self, _collection_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn update_asset(&self, _asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    pub async fn get_nft_assets_by_wallet_id(&self, device_id: i32, wallet_id: i32) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_id, wallet_id)?;

        let mut asset_ids: HashSet<NFTAssetId> = HashSet::new();
        for (sub, addr) in subscriptions {
            let chain = sub.chain.0;
            let ids = match self.get_provider_asset_ids(chain, &addr.address).await {
                Ok(ids) => ids,
                Err(_) => self.get_cached_asset_ids(addr.id, chain)?,
            };
            asset_ids.extend(ids);
        }

        self.preload(asset_ids.into_iter().collect()).await
    }

    async fn get_provider_asset_ids(&self, chain: Chain, address: &str) -> Result<HashSet<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .provider_client
            .get_nft_data(chain, address)
            .await?
            .into_iter()
            .flat_map(|d| d.assets.into_iter().filter_map(|a| NFTAssetId::from_id(&a.id)))
            .collect())
    }

    fn get_cached_asset_ids(&self, address_id: i32, chain: Chain) -> Result<HashSet<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .nft()?
            .get_nft_assets_by_filter(vec![NftAssetFilter::AddressId(address_id)])?
            .into_iter()
            .filter(|row| row.chain.0 == chain)
            .filter_map(|row| NFTAssetId::from_id(&row.identifier))
            .collect())
    }

    fn with_urls_asset(&self, asset: NFTAsset) -> NFTAsset {
        let preview_url = ImageFormatter::get_nft_asset_url(&self.assets_url, &asset.id);
        let resource_url = ImageFormatter::get_nft_asset_resource_url(&self.assets_url, &asset.id);
        asset.with_urls(preview_url, resource_url)
    }

    fn with_urls_collection(&self, collection: NFTCollection) -> NFTCollection {
        let preview_url = ImageFormatter::get_nft_collection_url(&self.assets_url, &collection.id);
        collection.with_preview_url(preview_url)
    }

    fn with_urls_data(&self, data: NFTData) -> NFTData {
        NFTData {
            collection: self.with_urls_collection(data.collection),
            assets: data.assets.into_iter().map(|a| self.with_urls_asset(a)).collect(),
        }
    }

    pub async fn preload(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let collection_ids: HashSet<NFTCollectionId> = assets.iter().map(|x| x.get_collection_id()).collect();
        let collection_id_map = self.preload_collections(collection_ids.into_iter().collect()).await?;
        self.preload_assets(assets.clone(), &collection_id_map).await?;
        self.get_nfts(assets).await
    }

    pub async fn preload_collections(&self, collection_ids: Vec<NFTCollectionId>) -> Result<HashMap<String, i32>, Box<dyn Error + Send + Sync>> {
        let identifiers: Vec<String> = collection_ids.iter().map(|x| x.id()).collect();
        let existing = self.get_nft_collection_id_map(&identifiers)?;

        let mut new_collections: Vec<NFTCollection> = Vec::new();
        for id in collection_ids.into_iter().filter(|id| !existing.contains_key(&id.id())) {
            if let Ok(collection) = self.provider_client.get_nft_collection(id).await {
                new_collections.push(collection);
            }
        }

        if new_collections.is_empty() {
            return Ok(existing);
        }

        let rows = new_collections.iter().cloned().map(NewNftCollectionRow::from_primitive).collect();
        self.database.nft()?.add_nft_collections(rows)?;

        let map = self.get_nft_collection_id_map(&identifiers)?;

        let links: Vec<NftLinkRow> = new_collections
            .into_iter()
            .flat_map(|collection| {
                let pk = map.get(&collection.id).copied();
                collection
                    .links
                    .into_iter()
                    .filter(|link| !link.url.is_empty())
                    .filter_map(move |link| pk.map(|pk| NftLinkRow::from_primitive(pk, link)))
            })
            .collect();
        self.database.nft()?.add_nft_collections_links(links)?;

        Ok(map)
    }

    pub async fn preload_assets(&self, asset_ids: Vec<NFTAssetId>, collection_id_map: &HashMap<String, i32>) -> Result<HashMap<String, i32>, Box<dyn Error + Send + Sync>> {
        let identifiers: Vec<String> = asset_ids.iter().map(|x| x.to_string()).collect();
        let existing = self.get_nft_asset_id_map(&identifiers)?;

        let mut new_assets: Vec<NFTAsset> = Vec::new();
        for id in asset_ids.into_iter().filter(|id| !existing.contains_key(&id.to_string())) {
            if let Ok(asset) = self.provider_client.get_nft_asset(id).await {
                new_assets.push(asset);
            }
        }

        let rows: Vec<NewNftAssetRow> = new_assets
            .into_iter()
            .filter_map(|asset| collection_id_map.get(&asset.collection_id).map(|&pk| NewNftAssetRow::from_primitive(asset, pk)))
            .collect();

        if rows.is_empty() {
            return Ok(existing);
        }

        self.database.nft()?.add_nft_assets(rows)?;
        self.get_nft_asset_id_map(&identifiers)
    }

    fn get_nft_collection_id_map(&self, identifiers: &[String]) -> Result<HashMap<String, i32>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .nft()?
            .get_nft_collections_by_filter(vec![NftCollectionFilter::Identifiers(identifiers.to_vec())])?
            .into_iter()
            .map(|c| (c.identifier, c.id))
            .collect())
    }

    fn get_nft_asset_id_map(&self, identifiers: &[String]) -> Result<HashMap<String, i32>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .nft()?
            .get_nft_assets_by_filter(vec![NftAssetFilter::Identifiers(identifiers.to_vec())])?
            .into_iter()
            .map(|a| (a.identifier, a.id))
            .collect())
    }

    fn load_nft_assets(&self, asset_identifiers: Vec<String>) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let assets = self.database.nft()?.get_nft_assets_by_filter(vec![NftAssetFilter::Identifiers(asset_identifiers)])?;
        let collection_ids: Vec<i32> = assets.iter().map(|a| a.collection_id).collect::<HashSet<_>>().into_iter().collect();
        let collection_identifiers: HashMap<i32, String> = self
            .database
            .nft()?
            .get_nft_collections_by_filter(vec![NftCollectionFilter::Ids(collection_ids)])?
            .into_iter()
            .map(|c| (c.id, c.identifier))
            .collect();
        Ok(assets
            .into_iter()
            .map(|row| {
                let identifier = collection_identifiers.get(&row.collection_id).cloned().unwrap_or_default();
                row.as_primitive(identifier)
            })
            .collect())
    }

    async fn get_nfts(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let mut by_collection: HashMap<NFTCollectionId, Vec<NFTAssetId>> = HashMap::new();
        for asset in assets {
            by_collection.entry(asset.get_collection_id()).or_default().push(asset);
        }

        by_collection
            .into_iter()
            .map(|(collection_id, asset_ids)| {
                let collection = self.load_nft_collection(&collection_id.id())?;
                let assets = self.load_nft_assets(asset_ids.into_iter().map(|x| x.to_string()).collect())?;
                Ok(self.with_urls_data(NFTData { collection, assets }))
            })
            .collect()
    }

    pub fn load_nft_asset(&self, asset_id: &str) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        self.load_nft_assets(vec![asset_id.to_string()])?.into_iter().next().ok_or_else(|| "asset not found".into())
    }

    pub fn load_nft_collection(&self, collection_id: &str) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let row = self.database.nft()?.get_nft_collection(collection_id)?;
        let links = self.database.nft()?.get_nft_collection_links(row.id)?.into_iter().map(|x| x.as_primitive()).collect();
        Ok(row.as_primitive(links))
    }

    pub async fn update_assets_for_addresses(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let address_id_map: HashMap<String, i32> = self
            .database
            .wallets()?
            .get_addresses(addresses.values().cloned().collect())?
            .into_iter()
            .map(|row| (row.address, row.id))
            .collect();

        let mut all_asset_ids: HashSet<NFTAssetId> = HashSet::new();
        let mut owned_by_address: HashMap<i32, HashSet<NFTAssetId>> = HashMap::new();
        let mut chains_by_address: HashMap<i32, HashSet<Chain>> = HashMap::new();
        for (chain, address) in addresses {
            let Ok(ids) = self.get_provider_asset_ids(chain, &address).await else { continue };
            if let Some(&address_id) = address_id_map.get(&address) {
                chains_by_address.entry(address_id).or_default().insert(chain);
                owned_by_address.entry(address_id).or_default().extend(ids.iter().cloned());
            }
            all_asset_ids.extend(ids);
        }

        let asset_ids: Vec<NFTAssetId> = all_asset_ids.into_iter().collect();
        let collection_ids: Vec<NFTCollectionId> = asset_ids.iter().map(|x| x.get_collection_id()).collect::<HashSet<_>>().into_iter().collect();
        let collection_id_map = self.preload_collections(collection_ids).await?;
        let asset_id_map = self.preload_assets(asset_ids.clone(), &collection_id_map).await?;

        for (address_id, owned) in owned_by_address {
            let current_asset_ids: Vec<i32> = owned.into_iter().filter_map(|id| asset_id_map.get(&id.to_string()).copied()).collect();
            let chains: Vec<Chain> = chains_by_address.remove(&address_id).unwrap_or_default().into_iter().collect();
            self.database.nft()?.set_nft_asset_associations(address_id, chains, current_asset_ids)?;
        }

        self.get_nfts(asset_ids).await
    }

    pub fn report_nft(&self, device_id: &str, collection_id: String, asset_id: Option<AssetId>, reason: Option<String>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut client = self.database.client()?;
        let device = DevicesStore::get_device(&mut client, device_id)?;
        let collection_pk = client.nft().get_nft_collection(&collection_id)?.id;
        let asset_pk = asset_id.and_then(|id| client.nft().get_nft_asset(&id.to_string()).ok().map(|row| row.id));

        client.nft().add_nft_report(storage::models::NewNftReportRow {
            device_id: device.id,
            collection_id: collection_pk,
            asset_id: asset_pk,
            reason,
        })?;
        Ok(true)
    }
}
