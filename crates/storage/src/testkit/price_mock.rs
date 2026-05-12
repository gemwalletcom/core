use crate::models::PriceRow;
use crate::sql_types::{PriceId, PriceProviderRow};
use chrono::Utc;
use primitives::{PriceId as PrimitivePriceId, PriceProvider};

impl PriceRow {
    pub fn mock(provider: PriceProvider, provider_price_id: &str) -> Self {
        Self {
            id: PriceId::from(PrimitivePriceId::new(provider, provider_price_id.to_string())),
            provider: PriceProviderRow(provider),
            price: 1.0,
            price_change_percentage_24h: None,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap_rank: None,
            total_volume: None,
            last_updated_at: Utc::now().naive_utc(),
        }
    }
}
