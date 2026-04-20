use crate::{Asset, AssetLink, AssetMarket, PriceAlert, PriceProvider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTime<Utc>,
    #[typeshare(skip)]
    pub provider: PriceProvider,
}

impl Price {
    pub fn new(price: f64, price_change_percentage_24h: f64, updated_at: DateTime<Utc>, provider: PriceProvider) -> Self {
        Price {
            price,
            price_change_percentage_24h,
            updated_at,
            provider,
        }
    }

    pub fn with_rate(self, rate: f64) -> Self {
        Price { price: self.price * rate, ..self }
    }

    pub fn new_with_rate(&self, base_rate: f64, rate: f64) -> Self {
        self.with_rate(rate * base_rate)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Equatable")]
struct PriceData {
    asset: Asset,
    price: Option<Price>,
    price_alerts: Vec<PriceAlert>,
    market: Option<AssetMarket>,
    links: Vec<AssetLink>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_rate() {
        let price = Price::new(100.0, 5.0, DateTime::default(), PriceProvider::Coingecko);

        let new_price = price.new_with_rate(1.0, 2.0);
        assert_eq!(new_price.price, 200.0);
        assert_eq!(new_price.price_change_percentage_24h, 5.0);

        let new_price = price.new_with_rate(2.0, 1.0);
        assert_eq!(new_price.price, 200.0);
        assert_eq!(new_price.price_change_percentage_24h, 5.0);

        let new_price = price.new_with_rate(1.0, 0.5);
        assert_eq!(new_price.price, 50.0);
        assert_eq!(new_price.price_change_percentage_24h, 5.0);
    }
}
