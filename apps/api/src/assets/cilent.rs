use std::error::Error;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, ChainAddress, NFTCollection, Perpetual};
use search_index::{ASSETS_INDEX_NAME, AssetDocument, NFTDocument, NFTS_INDEX_NAME, PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};
use storage::Database;

#[derive(Clone)]
pub struct AssetsClient {
    database: Database,
}

impl AssetsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn add_assets(&self, assets: Vec<Asset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = assets.into_iter().map(|x| x.as_basic_primitive()).collect();
        self.database
            .client()?
            .assets()
            .add_assets(assets)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    #[allow(unused)]
    pub fn get_asset(&self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.assets().get_asset(asset_id)?)
    }

    pub fn get_assets(&self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.assets().get_assets_basic(asset_ids)?)
    }

    pub fn get_asset_full(&self, asset_id: &str) -> Result<AssetFull, Box<dyn Error + Send + Sync>> {
        Ok(self.database.client()?.assets().get_asset_full(asset_id)?)
    }

    pub fn get_assets_by_device_id(
        &self,
        device_id: &str,
        wallet_index: i32,
        from_timestamp: Option<u32>,
    ) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self
            .database
            .client()?
            .subscriptions()
            .get_subscriptions_by_device_id(device_id, Some(wallet_index))?;

        let chain_addresses = subscriptions.into_iter().map(|x| ChainAddress::new(x.chain, x.address)).collect();

        Ok(self
            .database
            .client()?
            .assets_addresses()
            .get_assets_by_addresses(chain_addresses, from_timestamp, true)?)
    }
}

pub struct SearchRequest {
    pub query: String,
    pub chains: Vec<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub offset: usize,
}

pub struct SearchClient {
    client: SearchIndexClient,
}

impl SearchClient {
    pub async fn new(client: &SearchIndexClient) -> Self {
        Self { client: client.clone() }
    }

    pub async fn get_assets_search(&self, request: &SearchRequest) -> Result<Vec<primitives::AssetBasic>, Box<dyn Error + Send + Sync>> {
        let mut filters = vec![];
        filters.push("score.rank > 0".to_string());
        //filters.push("properties.isEnabled = true".to_string()); // Does not work, why?

        if !request.tags.is_empty() {
            filters.push(filter_array("tags", request.tags.clone()));
        }

        if !request.chains.is_empty() {
            filters.push(filter_array("asset.chain", request.chains.clone()));
        }

        let assets: Vec<AssetDocument> = self
            .client
            .search(
                ASSETS_INDEX_NAME,
                &request.query,
                &build_filter(filters),
                [].as_ref(),
                request.limit,
                request.offset,
            )
            .await?;

        Ok(assets.into_iter().map(|x| AssetBasic::new(x.asset, x.properties, x.score)).collect())
    }

    pub async fn get_perpetuals_search(&self, request: &SearchRequest) -> Result<Vec<Perpetual>, Box<dyn Error + Send + Sync>> {
        let perpetuals: Vec<PerpetualDocument> = self
            .client
            .search(
                PERPETUALS_INDEX_NAME,
                &request.query,
                &build_filter(vec![]),
                [].as_ref(),
                request.limit,
                request.offset,
            )
            .await?;

        Ok(perpetuals.into_iter().map(|x| x.perpetual).collect())
    }

    pub async fn get_nfts_search(&self, request: &SearchRequest) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let nfts: Vec<NFTDocument> = self
            .client
            .search(
                NFTS_INDEX_NAME,
                &request.query,
                &build_filter(vec![]),
                [].as_ref(),
                request.limit,
                request.offset,
            )
            .await?;

        Ok(nfts.into_iter().map(|x| x.collection).collect())
    }
}

fn build_filter(filters: Vec<String>) -> String {
    filters.join(" AND ")
}

fn filter_array(field: &str, values: Vec<String>) -> String {
    format!("{} IN [\"{}\"]", field, values.join("\",\""))
}
