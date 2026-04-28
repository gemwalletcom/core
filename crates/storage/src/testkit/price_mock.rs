use crate::models::PriceRow;
use crate::sql_types::PriceProviderRow;
use chrono::Utc;
use primitives::{PriceId, PriceProvider};

impl PriceRow {
    pub fn mock(provider: PriceProvider, provider_price_id: &str) -> Self {
        Self {
            id: PriceId::id_for(provider, provider_price_id),
            provider: PriceProviderRow(provider),
            provider_price_id: provider_price_id.to_string(),
            price: 1.0,
            price_change_percentage_24h: 0.0,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap_rank: None,
            last_updated_at: Utc::now().naive_utc(),
        }
    }
}
