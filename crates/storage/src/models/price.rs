use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetMarket, AssetPriceInfo};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use super::{Asset, CreateChart};

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Price {
    pub id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub all_time_high: f64,
    pub all_time_high_date: Option<NaiveDateTime>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<NaiveDateTime>,
    pub market_cap: f64,
    pub market_cap_fdv: f64,
    pub market_cap_rank: i32,
    pub total_volume: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub max_supply: f64,
    pub last_updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAsset {
    pub asset_id: String,
    pub price_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable)]
pub struct PriceAssetData {
    pub asset: Asset,
    pub price: Option<Price>,
}

impl PartialEq for PriceAsset {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id && self.price_id == other.price_id
    }
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Price {}

impl Hash for Price {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Price {
    pub fn new(
        id: String,
        price: f64,
        price_change_percentage_24h: f64,
        all_time_high: f64,
        all_time_high_date: Option<NaiveDateTime>,
        all_time_low: f64,
        all_time_low_date: Option<NaiveDateTime>,
        market_cap: f64,
        market_cap_fdv: f64,
        market_cap_rank: i32,
        total_volume: f64,
        circulating_supply: f64,
        total_supply: f64,
        max_supply: f64,
        last_updated_at: Option<NaiveDateTime>,
    ) -> Self {
        Price {
            id,
            price,
            price_change_percentage_24h,
            last_updated_at,
            all_time_high,
            all_time_high_date,
            all_time_low,
            all_time_low_date,
            market_cap,
            market_cap_fdv,
            market_cap_rank,
            total_volume,
            circulating_supply,
            total_supply,
            max_supply,
        }
    }

    pub fn for_rate(price: Price, base_rate: f64, rate: primitives::FiatRate) -> Price {
        let mut new_price = price.clone();
        let rate_multiplier = rate.multiplier(base_rate);
        new_price.price = price.price * rate_multiplier;
        new_price.market_cap = price.market_cap * rate_multiplier;
        new_price.market_cap_fdv = price.market_cap_fdv * rate_multiplier;
        new_price.total_volume = price.total_volume * rate_multiplier;
        new_price
    }
}

impl Price {
    pub fn as_price_primitive(&self) -> primitives::Price {
        primitives::Price::new(self.price, self.price_change_percentage_24h, self.last_updated_at)
    }

    pub fn as_market_primitive(&self) -> AssetMarket {
        AssetMarket {
            market_cap: Some(self.market_cap),
            market_cap_fdv: Some(self.market_cap_fdv),
            market_cap_rank: Some(self.market_cap_rank),
            total_volume: self.total_volume.into(),
            circulating_supply: Some(self.circulating_supply),
            total_supply: Some(self.total_supply),
            max_supply: Some(self.max_supply),
        }
    }

    pub fn as_chart(&self) -> CreateChart {
        CreateChart {
            coin_id: self.id.clone(),
            price: self.price as f32,
            ts: self.last_updated_at.unwrap_or_default().and_utc().timestamp() as u32,
        }
    }

    pub fn as_price_asset_info(&self, asset_id: &str) -> AssetPriceInfo {
        AssetPriceInfo {
            asset_id: asset_id.to_string(),
            price: self.as_price_primitive(),
            market: self.as_market_primitive(),
        }
    }
}
