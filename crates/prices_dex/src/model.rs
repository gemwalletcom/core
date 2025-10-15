use chrono::{DateTime, Utc};
use primitives::{AssetId, PriceFeedId};

#[derive(Debug, Clone)]
pub struct DexAssetPrice {
    pub asset_id: AssetId,
    pub price_feed: AssetPriceFeed,
    pub price: f64,
    pub updated_at: DateTime<Utc>,
}

impl DexAssetPrice {
    pub fn new(asset_id: AssetId, price_feed: AssetPriceFeed, price: f64, updated_at: DateTime<Utc>) -> Self {
        Self {
            asset_id,
            price_feed,
            price,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetPriceFeed {
    pub price_feed_id: PriceFeedId,
    pub asset_id: AssetId,
}

impl AssetPriceFeed {
    pub fn new(price_feed_id: PriceFeedId, asset_id: AssetId) -> Self {
        Self { price_feed_id, asset_id }
    }

    pub fn get_id(&self) -> String {
        self.price_feed_id.get_id()
    }
}
