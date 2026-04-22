use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId as PrimitiveAssetId, AssetMarket, AssetPriceInfo, AssetPriceKey, ChartValuePercentage, FiatRate, Price, PriceData, PriceProvider};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::sql_types::{AssetId, PriceProviderRow};

use super::AssetRow;

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceRow {
    pub id: String,
    pub provider: PriceProviderRow,
    pub provider_price_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub all_time_high: f64,
    pub all_time_high_date: Option<NaiveDateTime>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<NaiveDateTime>,
    pub market_cap_rank: Option<i32>,
    pub last_updated_at: NaiveDateTime,
}

#[derive(Debug, Selectable, Identifiable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::prices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPriceRow {
    pub id: String,
    pub provider: PriceProviderRow,
    pub provider_price_id: String,
}

impl NewPriceRow {
    pub fn new(provider: PriceProvider, provider_price_id: String) -> Self {
        let id = AssetPriceKey::id_for(provider, &provider_price_id);
        Self {
            id,
            provider: provider.into(),
            provider_price_id,
        }
    }
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAssetRow {
    pub asset_id: AssetId,
    pub price_id: String,
    pub provider: PriceProviderRow,
}

impl PriceAssetRow {
    pub fn new(asset_id: PrimitiveAssetId, provider: PriceProvider, provider_price_id: &str) -> Self {
        PriceAssetRow {
            asset_id: asset_id.into(),
            price_id: AssetPriceKey::id_for(provider, provider_price_id),
            provider: provider.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable)]
pub struct PriceAssetDataRow {
    pub asset: AssetRow,
    pub price: Option<PriceRow>,
}

impl PartialEq for PriceAssetRow {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id && self.price_id == other.price_id
    }
}

impl PartialEq for PriceRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for PriceRow {}

impl Hash for PriceRow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PriceRow {
    pub fn with_price(provider: PriceProvider, provider_price_id: String, price: f64) -> Self {
        Self::new(provider, provider_price_id, price, 0.0, 0.0, None, 0.0, None, None, chrono::Utc::now().naive_utc())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: PriceProvider,
        provider_price_id: String,
        price: f64,
        price_change_percentage_24h: f64,
        all_time_high: f64,
        all_time_high_date: Option<NaiveDateTime>,
        all_time_low: f64,
        all_time_low_date: Option<NaiveDateTime>,
        market_cap_rank: Option<i32>,
        last_updated_at: NaiveDateTime,
    ) -> Self {
        let id = AssetPriceKey::id_for(provider, &provider_price_id);
        PriceRow {
            id,
            provider: provider.into(),
            provider_price_id,
            price,
            price_change_percentage_24h,
            last_updated_at,
            all_time_high,
            all_time_high_date,
            all_time_low,
            all_time_low_date,
            market_cap_rank,
        }
    }

    pub fn for_rate(price: PriceRow, base_rate: f64, rate: FiatRate) -> PriceRow {
        let mut new_price = price.clone();
        let rate_multiplier = rate.multiplier(base_rate);
        new_price.price = price.price * rate_multiplier;
        new_price.all_time_high = price.all_time_high * rate_multiplier;
        new_price.all_time_low = price.all_time_low * rate_multiplier;
        new_price
    }

    pub fn provider_value(&self) -> PriceProvider {
        self.provider.0
    }
}

impl PriceRow {
    pub fn as_primitive(&self) -> Price {
        Price::new(self.price, self.price_change_percentage_24h, self.last_updated_at.and_utc(), self.provider_value())
    }

    pub fn as_market_primitive(&self, asset: &AssetRow) -> AssetMarket {
        let ath_percentage = if self.all_time_high > 0.0 {
            Some((self.price - self.all_time_high) / self.all_time_high * 100.0)
        } else {
            None
        };
        let atl_percentage = if self.all_time_low > 0.0 {
            Some((self.price - self.all_time_low) / self.all_time_low * 100.0)
        } else {
            None
        };
        let market_cap = asset.circulating_supply.map(|supply| self.price * supply);
        let market_cap_fdv = asset.total_supply.or(asset.max_supply).map(|supply| self.price * supply);
        AssetMarket {
            market_cap,
            market_cap_fdv,
            market_cap_rank: self.market_cap_rank,
            total_volume: None,
            circulating_supply: asset.circulating_supply,
            total_supply: asset.total_supply,
            max_supply: asset.max_supply,
            all_time_high: Some(self.all_time_high),
            all_time_high_date: self.all_time_high_date.map(|d| d.and_utc()),
            all_time_high_change_percentage: ath_percentage,
            all_time_low: Some(self.all_time_low),
            all_time_low_date: self.all_time_low_date.map(|d| d.and_utc()),
            all_time_low_change_percentage: atl_percentage,
            all_time_high_value: self.all_time_high_date.map(|d| ChartValuePercentage {
                date: d.and_utc(),
                value: self.all_time_high as f32,
                percentage: ath_percentage.unwrap_or_default() as f32,
            }),
            all_time_low_value: self.all_time_low_date.map(|d| ChartValuePercentage {
                date: d.and_utc(),
                value: self.all_time_low as f32,
                percentage: atl_percentage.unwrap_or_default() as f32,
            }),
        }
    }

    pub fn as_price_asset_info(&self, asset: &AssetRow) -> AssetPriceInfo {
        AssetPriceInfo {
            asset_id: asset.as_asset_id(),
            price: self.as_primitive(),
            market: self.as_market_primitive(asset),
        }
    }

    pub fn as_price_data(&self) -> PriceData {
        PriceData {
            id: self.id.clone(),
            provider: self.provider_value(),
            provider_price_id: self.provider_price_id.clone(),
            price: self.price,
            price_change_percentage_24h: self.price_change_percentage_24h,
            all_time_high: self.all_time_high,
            all_time_high_date: self.all_time_high_date.map(|d| d.and_utc()),
            all_time_low: self.all_time_low,
            all_time_low_date: self.all_time_low_date.map(|d| d.and_utc()),
            market_cap_rank: self.market_cap_rank,
            last_updated_at: self.last_updated_at.and_utc(),
        }
    }

    pub fn from_price_data(data: PriceData) -> Self {
        PriceRow {
            id: data.id,
            provider: data.provider.into(),
            provider_price_id: data.provider_price_id,
            price: data.price,
            price_change_percentage_24h: data.price_change_percentage_24h,
            all_time_high: data.all_time_high,
            all_time_high_date: data.all_time_high_date.map(|d| d.naive_utc()),
            all_time_low: data.all_time_low,
            all_time_low_date: data.all_time_low_date.map(|d| d.naive_utc()),
            market_cap_rank: data.market_cap_rank,
            last_updated_at: data.last_updated_at.naive_utc(),
        }
    }
}
