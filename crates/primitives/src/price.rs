use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Asset, AssetLink, AssetMarket, PriceAlert};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTime<Utc>,
}

impl Price {
    pub fn new(price: f64, price_change_percentage_24h: f64, updated_at: DateTime<Utc>) -> Self {
        Price {
            price,
            price_change_percentage_24h,
            updated_at,
        }
    }

    pub fn new_with_rate(&self, base_rate: f64, rate: f64) -> Self {
        let rate_multiplier = rate * base_rate;
        let price_value = self.price * rate_multiplier;

        Price {
            price: price_value,
            price_change_percentage_24h: self.price_change_percentage_24h,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceFull {
    pub asset_id: String,
    pub coin_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
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
        let price = Price {
            price: 100.0,
            price_change_percentage_24h: 5.0,
            updated_at: Utc::now(),
        };

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
