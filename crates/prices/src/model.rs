use chrono::Utc;
use primitives::{AssetId, AssetLink, AssetMarket, Price, PriceData, PriceId, PriceProvider};

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
pub struct PriceProviderAsset {
    pub mapping: AssetPriceMapping,
    pub market: Option<AssetMarket>,
    pub price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
}

impl PriceProviderAsset {
    pub fn new(mapping: AssetPriceMapping, market: Option<AssetMarket>) -> Self {
        Self {
            mapping,
            market,
            price: None,
            price_change_percentage_24h: None,
        }
    }

    pub fn with_price(mapping: AssetPriceMapping, market: Option<AssetMarket>, price: Option<f64>, price_change_percentage_24h: Option<f64>) -> Self {
        Self {
            mapping,
            market,
            price,
            price_change_percentage_24h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriceProviderAssetMetadata {
    pub asset_id: AssetId,
    pub rank: i32,
    pub links: Vec<AssetLink>,
}

impl PriceProviderAssetMetadata {
    pub fn new(asset_id: AssetId, rank: i32, links: Vec<AssetLink>) -> Self {
        Self { asset_id, rank, links }
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

    pub fn from_provider_asset(asset: PriceProviderAsset, provider: PriceProvider) -> Self {
        Self::new(
            asset.mapping,
            Price::new(asset.price.unwrap_or_default(), asset.price_change_percentage_24h.unwrap_or_default(), Utc::now(), provider),
            asset.market,
        )
    }

    pub fn as_price_data(&self) -> PriceData {
        let market = self.market.clone().unwrap_or_default();
        PriceData {
            id: PriceId::id_for(self.price.provider, &self.mapping.provider_price_id),
            provider: self.price.provider,
            provider_price_id: self.mapping.provider_price_id.clone(),
            price: self.price.price,
            price_change_percentage_24h: self.price.price_change_percentage_24h,
            all_time_high: market.all_time_high.unwrap_or_default(),
            all_time_high_date: market.all_time_high_date,
            all_time_low: market.all_time_low.unwrap_or_default(),
            all_time_low_date: market.all_time_low_date,
            market_cap_rank: market.market_cap_rank,
            last_updated_at: self.price.updated_at,
        }
    }
}
