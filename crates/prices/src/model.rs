use chrono::Utc;
use primitives::{AssetId, AssetMarket, Price, PriceProvider};

#[derive(Debug, Clone)]
pub struct AssetPriceMapping {
    pub asset_id: AssetId,
    pub provider_price_id: String,
}

impl AssetPriceMapping {
    pub fn new(asset_id: AssetId, provider_price_id: String) -> Self {
        Self { asset_id, provider_price_id }
    }
}

#[derive(Debug, Clone)]
pub struct AssetPriceFull {
    pub mapping: AssetPriceMapping,
    pub price: Price,
    pub market: Option<AssetMarket>,
}

impl AssetPriceFull {
    pub fn new(mapping: AssetPriceMapping, price: Price, market: Option<AssetMarket>) -> Self {
        Self { mapping, price, market }
    }

    pub fn simple(mapping: AssetPriceMapping, price: f64, price_change_percentage_24h: f64, provider: PriceProvider) -> Self {
        Self::new(mapping, Price::new(price, price_change_percentage_24h, Utc::now(), provider), None)
    }
}
