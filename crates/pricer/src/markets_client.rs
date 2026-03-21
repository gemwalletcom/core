use std::error::Error;

use cacher::{CacheError, CacheKey, CacherClient};
use primitives::{AssetId, AssetTag, Markets, MarketsAssets};
use storage::{Database, PricesRepository, TagRepository};

#[derive(Clone)]
pub struct MarketsClient {
    database: Database,
    cacher: CacherClient,
}

impl MarketsClient {
    pub fn new(database: Database, cacher: CacherClient) -> Self {
        Self { database, cacher }
    }

    pub async fn get_markets(&self) -> Result<Markets, Box<dyn Error + Send + Sync>> {
        match self.cacher.get_cached_optional(CacheKey::Markets).await? {
            Some(markets) => Ok(markets),
            None => Err(Box::new(CacheError::not_found_resource("Markets"))),
        }
    }

    pub async fn set_markets(&self, markets: Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.set_cached(CacheKey::Markets, &markets).await
    }

    pub async fn get_asset_ids_for_prices_ids(&self, price_ids: Vec<String>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let assets = self.database.prices()?.get_prices_assets_for_price_ids(price_ids.clone())?;
        let asset_map: std::collections::HashMap<_, _> = assets.into_iter().map(|x| (x.price_id, x.asset_id)).collect();
        Ok(price_ids
            .into_iter()
            .filter_map(|price_id| asset_map.get(&price_id).map(|asset_id| asset_id.0.clone()))
            .collect())
    }

    pub fn set_asset_ids_for_tag(&self, tag: AssetTag, asset_ids: Vec<AssetId>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.tag()?.set_assets_tags_for_tag(tag.as_ref(), asset_ids)?)
    }

    pub fn get_asset_ids_for_tag(&self, tag: AssetTag) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.tag()?.get_assets_tags_for_tag(tag.as_ref())?.into_iter().map(|x| x.asset_id.0).collect())
    }

    pub fn get_market_assets(&self) -> Result<MarketsAssets, Box<dyn Error + Send + Sync>> {
        let assets = MarketsAssets {
            trending: self.get_asset_ids_for_tag(AssetTag::Trending)?,
            gainers: self.get_asset_ids_for_tag(AssetTag::Gainers)?,
            losers: self.get_asset_ids_for_tag(AssetTag::Losers)?,
        };
        Ok(assets)
    }
}
