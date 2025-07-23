use std::error::Error;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, ChainAddress};
use search_index::{AssetDocument, SearchIndexClient, ASSETS_INDEX_NAME};
use storage::DatabaseClient;

pub struct AssetsClient {
    database: Box<DatabaseClient>,
}

impl AssetsClient {
    pub async fn new(database_url: &str) -> Self {
        let database = Box::new(DatabaseClient::new(database_url));
        Self { database }
    }

    pub fn add_assets(&mut self, assets: Vec<Asset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = assets.into_iter().map(|x| x.as_basic_primitive()).collect();
        self.database.assets().add_assets(assets)
    }

    #[allow(unused)]
    pub fn get_asset(&mut self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.database.assets().get_asset(asset_id)
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        self.database.assets().get_assets_basic(asset_ids)
    }

    pub fn get_asset_full(&mut self, asset_id: &str) -> Result<AssetFull, Box<dyn Error + Send + Sync>> {
        self.database.assets().get_asset_full(asset_id)
    }

    pub fn get_assets_by_device_id(
        &mut self,
        device_id: &str,
        wallet_index: i32,
        from_timestamp: Option<u32>,
    ) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.subscriptions().get_subscriptions_by_device_id(device_id, Some(wallet_index))?;

        let chain_addresses = subscriptions.into_iter().map(|x| ChainAddress::new(x.chain, x.address)).collect();

        self.database.assets_addresses().get_assets_by_addresses(chain_addresses, from_timestamp, true)
    }
}

pub struct AssetsSearchClient {
    client: SearchIndexClient,
}

impl AssetsSearchClient {
    pub async fn new(client: &SearchIndexClient) -> Self {
        Self { client: client.clone() }
    }

    pub async fn get_assets_search(
        &mut self,
        query: &str,
        chains: Vec<String>,
        tags: Vec<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<primitives::AssetBasic>, Box<dyn Error + Send + Sync>> {
        let mut filters = vec![];
        filters.push("score.rank > 0".to_string());
        //filters.push("properties.isEnabled = true".to_string()); // Does not work, why?

        if !tags.is_empty() {
            filters.push(filter_array("tags", tags));
        }

        if !chains.is_empty() {
            filters.push(filter_array("asset.chain", chains));
        }
        let filter = &filters.join(" AND ");

        let assets: Vec<AssetDocument> = self.client.search(ASSETS_INDEX_NAME, query, filter, [].as_ref(), limit, offset).await?;

        Ok(assets.into_iter().map(|x| AssetBasic::new(x.asset, x.properties, x.score)).collect())
    }
}

fn filter_array(field: &str, values: Vec<String>) -> String {
    format!("{} IN [\"{}\"]", field, values.join("\",\""))
}
