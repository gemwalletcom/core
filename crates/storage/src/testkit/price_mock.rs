use crate::models::PriceRow;
use crate::sql_types::PriceProviderRow;
use chrono::Utc;
use primitives::PriceProvider;

impl PriceRow {
    pub fn mock(provider: PriceProvider, provider_price_id: &str) -> Self {
        Self {
            id: provider.price_id(provider_price_id),
            provider: PriceProviderRow(provider),
            provider_price_id: provider_price_id.to_string(),
            price: 1.0,
            price_change_percentage_24h: 0.0,
            all_time_high: 0.0,
            all_time_high_date: None,
            all_time_low: 0.0,
            all_time_low_date: None,
            market_cap: 0.0,
            market_cap_fdv: 0.0,
            market_cap_rank: 0,
            total_volume: 0.0,
            circulating_supply: 0.0,
            total_supply: 0.0,
            max_supply: 0.0,
            last_updated_at: Utc::now().naive_utc(),
        }
    }
}
