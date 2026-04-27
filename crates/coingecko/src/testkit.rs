use chrono::{DateTime, Utc};

use crate::CoinMarket;

impl CoinMarket {
    pub fn mock() -> Self {
        Self::mock_with_id("bitcoin")
    }

    pub fn mock_with_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            symbol: "btc".to_string(),
            name: "Bitcoin".to_string(),
            current_price: Some(0.12),
            price_change_percentage_24h: Some(1.5),
            market_cap: Some(1000.0),
            fully_diluted_valuation: Some(2000.0),
            market_cap_rank: Some(100),
            market_cap_rank_with_rehypothecated: Some(99),
            total_volume: Some(10.0),
            circulating_supply: Some(10000.0),
            total_supply: Some(20000.0),
            max_supply: Some(30000.0),
            ath: Some(1.0),
            ath_date: Some(DateTime::<Utc>::UNIX_EPOCH),
            atl: Some(0.01),
            atl_date: Some(DateTime::<Utc>::UNIX_EPOCH),
            last_updated: Some(DateTime::<Utc>::UNIX_EPOCH),
        }
    }
}
