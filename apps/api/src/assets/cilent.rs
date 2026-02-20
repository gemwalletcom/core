use std::collections::HashMap;
use std::error::Error;

use super::filter::{build_assets_filters, build_filter};
use super::model::SearchRequest;
use chrono::{DateTime, Utc};
use pricer::PriceClient;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, ChainAddress, NFTCollection, PerpetualSearchData};
use search_index::{ASSETS_INDEX_NAME, AssetDocument, NFTDocument, NFTS_INDEX_NAME, PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient};
use storage::{AssetsAddressesRepository, AssetsRepository, Database, WalletsRepository};

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
        self.database.assets()?.add_assets(assets).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    #[allow(unused)]
    pub fn get_asset(&self, asset_id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.database.assets()?.get_asset(asset_id)?)
    }

    pub fn get_assets(&self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.assets()?.get_assets_basic(asset_ids)?)
    }

    pub fn get_asset_full(&self, asset_id: &str) -> Result<AssetFull, Box<dyn Error + Send + Sync>> {
        Ok(self.database.assets()?.get_asset_full(asset_id)?)
    }

    pub fn get_assets_by_wallet_id(&self, device_id: i32, wallet_id: i32, from_timestamp: Option<u64>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
        let chain_addresses: Vec<ChainAddress> = subscriptions.into_iter().map(|(sub, addr)| ChainAddress::new(sub.chain.0, addr.address)).collect();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));

        Ok(self.database.assets_addresses()?.get_assets_by_addresses(chain_addresses, from_datetime, true)?)
    }
}

pub struct SearchClient {
    client: SearchIndexClient,
    price_client: PriceClient,
}

impl SearchClient {
    pub fn new(client: &SearchIndexClient, price_client: PriceClient) -> Self {
        Self {
            client: client.clone(),
            price_client,
        }
    }

    pub async fn get_assets_search(&self, request: &SearchRequest) -> Result<Vec<primitives::AssetBasic>, Box<dyn Error + Send + Sync>> {
        let filters = build_assets_filters(request);

        let assets: Vec<AssetDocument> = self
            .client
            .search(ASSETS_INDEX_NAME, &request.query, &build_filter(filters), [].as_ref(), request.limit, request.offset)
            .await?;

        if assets.is_empty() {
            return Ok(vec![]);
        }

        let asset_ids: Vec<String> = assets.iter().map(|x| x.asset.id.to_string()).collect();
        let prices: HashMap<String, _> = self
            .price_client
            .get_cache_prices(asset_ids)
            .await?
            .into_iter()
            .map(|p| (p.asset_id.to_string(), p.as_price_primitive()))
            .collect();

        Ok(assets
            .into_iter()
            .map(|x| {
                let price = prices.get(&x.asset.id.to_string()).cloned();
                AssetBasic {
                    asset: x.asset,
                    properties: x.properties,
                    score: x.score,
                    price,
                }
            })
            .collect())
    }

    pub async fn get_perpetuals_search(&self, request: &SearchRequest) -> Result<Vec<PerpetualSearchData>, Box<dyn Error + Send + Sync>> {
        let perpetuals: Vec<PerpetualDocument> = self
            .client
            .search(PERPETUALS_INDEX_NAME, &request.query, &build_filter(vec![]), [].as_ref(), request.limit, request.offset)
            .await?;

        Ok(perpetuals.into_iter().map(Into::into).collect())
    }

    pub async fn get_nfts_search(&self, request: &SearchRequest) -> Result<Vec<NFTCollection>, Box<dyn Error + Send + Sync>> {
        let nfts: Vec<NFTDocument> = self
            .client
            .search(NFTS_INDEX_NAME, &request.query, &build_filter(vec![]), [].as_ref(), request.limit, request.offset)
            .await?;

        Ok(nfts.into_iter().map(|x| x.collection).collect())
    }
}
