use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId as PrimitiveAssetId, AssetMarket, AssetPriceInfo, AssetPriceKey, ChartValuePercentage, Price, PriceData, PriceProvider};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::database::prices::PriceUpdate;
use crate::models::min_max::MinMax;

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
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub all_time_high: f64,
    pub all_time_high_date: Option<NaiveDateTime>,
    pub all_time_low: f64,
    pub all_time_low_date: Option<NaiveDateTime>,
}

#[derive(Default, AsChangeset)]
#[diesel(table_name = crate::schema::prices)]
#[diesel(treat_none_as_null = false)]
pub struct PricesChangeset {
    all_time_high: Option<f64>,
    all_time_high_date: Option<Option<NaiveDateTime>>,
    all_time_low: Option<f64>,
    all_time_low_date: Option<Option<NaiveDateTime>>,
    price_change_percentage_24h: Option<f64>,
}

impl PricesChangeset {
    pub fn from_updates(updates: Vec<PriceUpdate>) -> Self {
        updates.into_iter().fold(Self::default(), |mut acc, update| {
            match update {
                PriceUpdate::AllTimeHigh { value, date } => {
                    acc.all_time_high = Some(value);
                    acc.all_time_high_date = Some(date);
                }
                PriceUpdate::AllTimeLow { value, date } => {
                    acc.all_time_low = Some(value);
                    acc.all_time_low_date = Some(date);
                }
                PriceUpdate::PriceChangePercentage24h(value) => {
                    acc.price_change_percentage_24h = Some(value);
                }
            }
            acc
        })
    }
}

impl NewPriceRow {
    pub fn new(provider: PriceProvider, provider_price_id: String) -> Self {
        Self::with_market_data(provider, provider_price_id, None, None, None)
    }

    pub fn with_market_data(
        provider: PriceProvider,
        provider_price_id: String,
        market: Option<&AssetMarket>,
        price: Option<f64>,
        price_change_percentage_24h: Option<f64>,
    ) -> Self {
        let id = AssetPriceKey::id_for(provider, &provider_price_id);
        Self {
            id,
            provider: provider.into(),
            provider_price_id,
            price: price.unwrap_or(0.0),
            price_change_percentage_24h: price_change_percentage_24h.unwrap_or(0.0),
            all_time_high: market.and_then(|m| m.all_time_high).unwrap_or(0.0),
            all_time_high_date: market.and_then(|m| m.all_time_high_date).map(|d| d.naive_utc()),
            all_time_low: market.and_then(|m| m.all_time_low).unwrap_or(0.0),
            all_time_low_date: market.and_then(|m| m.all_time_low_date).map(|d| d.naive_utc()),
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

    pub(crate) fn merge_extremes_from_charts(&self, extremes: MinMax<f64>) -> Vec<PriceUpdate> {
        let mut updates = Vec::new();
        if let Some(point) = extremes.max
            && point.value >= self.all_time_high
            && (point.value, Some(point.date)) != (self.all_time_high, self.all_time_high_date)
        {
            updates.push(PriceUpdate::AllTimeHigh {
                value: point.value,
                date: Some(point.date),
            });
        }
        if let Some(point) = extremes.min
            && point.value > 0.0
            && (self.all_time_low == 0.0 || point.value <= self.all_time_low)
            && (point.value, Some(point.date)) != (self.all_time_low, self.all_time_low_date)
        {
            updates.push(PriceUpdate::AllTimeLow {
                value: point.value,
                date: Some(point.date),
            });
        }
        updates
    }

    pub fn merge_extremes(&self, wire: Option<&PriceRow>) -> Vec<PriceUpdate> {
        let mut updates = Vec::new();

        let mut high = (self.all_time_high, self.all_time_high_date);
        if let Some(w) = wire
            && (w.all_time_high > high.0 || (w.all_time_high == high.0 && w.all_time_high_date.is_some() && w.all_time_high_date != high.1))
        {
            high = (w.all_time_high, w.all_time_high_date);
        }
        if self.price > high.0 {
            high = (self.price, Some(self.last_updated_at));
        }
        if high != (self.all_time_high, self.all_time_high_date) {
            updates.push(PriceUpdate::AllTimeHigh { value: high.0, date: high.1 });
        }

        let mut low = (self.all_time_low, self.all_time_low_date);
        if let Some(w) = wire
            && w.all_time_low > 0.0
            && (low.0 == 0.0 || w.all_time_low < low.0 || (w.all_time_low == low.0 && w.all_time_low_date.is_some() && w.all_time_low_date != low.1))
        {
            low = (w.all_time_low, w.all_time_low_date);
        }
        if self.price > 0.0 && (low.0 == 0.0 || self.price < low.0) {
            low = (self.price, Some(self.last_updated_at));
        }
        if low != (self.all_time_low, self.all_time_low_date) {
            updates.push(PriceUpdate::AllTimeLow { value: low.0, date: low.1 });
        }

        updates
    }

    pub fn provider_value(&self) -> PriceProvider {
        self.provider.0
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> NaiveDateTime {
        chrono::Utc.timestamp_opt(secs, 0).unwrap().naive_utc()
    }

    fn row(price: f64, ath: f64, ath_d: Option<NaiveDateTime>, atl: f64, atl_d: Option<NaiveDateTime>) -> PriceRow {
        PriceRow::new(PriceProvider::Pyth, "x".into(), price, 0.0, ath, ath_d, atl, atl_d, None, ts(1000))
    }

    #[test]
    fn test_merge_extremes() {
        let stored = row(50.0, 100.0, Some(ts(100)), 10.0, Some(ts(200)));
        assert!(stored.merge_extremes(None).is_empty());

        let stored = row(5.0, 100.0, Some(ts(100)), 10.0, Some(ts(200)));
        let updates = stored.merge_extremes(None);
        assert_eq!(updates.len(), 1);
        assert!(matches!(updates[0], PriceUpdate::AllTimeLow { value, .. } if value == 5.0));

        let stored = row(50.0, 80.0, Some(ts(100)), 10.0, Some(ts(200)));
        let wire = row(50.0, 150.0, Some(ts(900)), 0.0, None);
        let updates = stored.merge_extremes(Some(&wire));
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            PriceUpdate::AllTimeHigh { value, date } => {
                assert_eq!(*value, 150.0);
                assert_eq!(*date, Some(ts(900)));
            }
            _ => panic!("expected AllTimeHigh"),
        }

        let stored = row(50.0, 100.0, Some(ts(100)), 10.0, Some(ts(200)));
        let wire = row(50.0, 0.0, None, 0.0, None);
        assert!(stored.merge_extremes(Some(&wire)).is_empty());

        let stored = row(7.5, 100.0, Some(ts(100)), 0.0, None);
        let updates = stored.merge_extremes(None);
        assert_eq!(updates.len(), 1);
        assert!(matches!(updates[0], PriceUpdate::AllTimeLow { value, .. } if value == 7.5));

        let stored = row(200.0, 100.0, Some(ts(100)), 10.0, Some(ts(200)));
        let wire = row(200.0, 150.0, Some(ts(900)), 0.0, None);
        let updates = stored.merge_extremes(Some(&wire));
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            PriceUpdate::AllTimeHigh { value, date } => {
                assert_eq!(*value, 200.0);
                assert_eq!(*date, Some(ts(1000)));
            }
            _ => panic!("expected AllTimeHigh"),
        }
    }
}
