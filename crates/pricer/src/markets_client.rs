use std::error::Error;

use cacher::CacherClient;
use primitives::{AssetId, AssetTag, Markets, MarketsAssets};
use storage::DatabaseClient;

pub struct MarketsClient {
    database: DatabaseClient,
    cacher: CacherClient,
}

const MARKETS_KEY: &str = "markets";

impl MarketsClient {
    pub fn new(database_url: &str, cacher: CacherClient) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database, cacher }
    }

    pub async fn get_markets(&mut self) -> Result<Markets, Box<dyn Error + Send + Sync>> {
        self.cacher.get_value::<Markets>(MARKETS_KEY).await
    }

    pub async fn set_markets(&mut self, markets: Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.set_value(MARKETS_KEY, &markets).await
    }

    pub async fn get_asset_ids_for_prices_ids(&mut self, price_ids: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let assets = self.database.prices().get_prices_assets_for_price_ids(price_ids.clone())?;
        // use same order as price_ids
        let asset_map: std::collections::HashMap<_, _> = assets.into_iter().map(|asset| (asset.price_id, asset.asset_id)).collect();
        Ok(price_ids.into_iter().filter_map(|price_id| asset_map.get(&price_id).cloned()).collect())
    }

    pub fn set_asset_ids_for_tag(&mut self, tag: AssetTag, asset_ids: Vec<String>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.tag().set_assets_tags_for_tag(tag.as_ref(), asset_ids)?)
    }

    pub fn get_asset_ids_for_tag(&mut self, tag: AssetTag) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .tag()
            .get_assets_tags_for_tag(tag.as_ref())?
            .into_iter()
            .flat_map(|x| AssetId::new(x.asset_id.as_str()))
            .collect())
    }

    pub fn get_market_assets(&mut self) -> Result<MarketsAssets, Box<dyn Error + Send + Sync>> {
        let assets = MarketsAssets {
            trending: self.get_asset_ids_for_tag(AssetTag::Trending)?,
            gainers: self.get_asset_ids_for_tag(AssetTag::Gainers)?,
            losers: self.get_asset_ids_for_tag(AssetTag::Losers)?,
        };
        Ok(assets)
    }
}
