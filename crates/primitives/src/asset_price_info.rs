use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, AssetMarket, AssetPrice, Price};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
#[typeshare(skip)]
pub struct AssetPriceInfo {
    pub asset_id: AssetId,
    pub price: Price,
    pub market: AssetMarket,
}

impl AssetPriceInfo {
    pub fn as_price_primitive(&self) -> Price {
        Price::new(self.price.price, self.price.price_change_percentage_24h, self.price.updated_at)
    }

    pub fn as_price_primitive_with_rate(&self, rate: f64) -> Price {
        Price::new(self.price.price * rate, self.price.price_change_percentage_24h, self.price.updated_at)
    }

    pub fn as_asset_price_primitive(&self) -> AssetPrice {
        self.as_asset_price_primitive_with_rate(1.0)
    }

    pub fn as_asset_price_primitive_with_rate(&self, rate: f64) -> AssetPrice {
        AssetPrice {
            asset_id: self.asset_id.clone(),
            price: self.price.price * rate,
            price_change_percentage_24h: self.price.price_change_percentage_24h,
            updated_at: self.price.updated_at,
        }
    }

    pub fn as_market(&self) -> AssetMarket {
        self.as_market_with_rate(1.0)
    }

    pub fn as_market_with_rate(&self, rate: f64) -> AssetMarket {
        let current_price = self.price.price;
        AssetMarket {
            market_cap: self.market.market_cap.map(|x| x * rate),
            market_cap_fdv: self.market.market_cap_fdv.map(|x| x * rate),
            market_cap_rank: self.market.market_cap_rank,
            total_volume: self.market.total_volume.map(|x| x * rate),
            circulating_supply: self.market.circulating_supply,
            total_supply: self.market.total_supply,
            max_supply: self.market.max_supply,
            all_time_high: self.market.all_time_high.map(|x| x * rate),
            all_time_high_date: self.market.all_time_high_date,
            all_time_high_change_percentage: self.market.all_time_high.map(|ath| (current_price - ath) / ath * 100.0),
            all_time_low: self.market.all_time_low.map(|x| x * rate),
            all_time_low_date: self.market.all_time_low_date,
            all_time_low_change_percentage: self.market.all_time_low.map(|atl| (current_price - atl) / atl * 100.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;
    use chrono::Utc;

    #[test]
    fn test_all_time_change_percentage() {
        let info = AssetPriceInfo {
            asset_id: AssetId::from(Chain::Bitcoin, None),
            price: Price::new(80.0, 5.0, Utc::now()),
            market: AssetMarket {
                market_cap: None,
                market_cap_fdv: None,
                market_cap_rank: None,
                total_volume: None,
                circulating_supply: None,
                total_supply: None,
                max_supply: None,
                all_time_high: Some(100.0),
                all_time_high_date: None,
                all_time_high_change_percentage: None,
                all_time_low: Some(40.0),
                all_time_low_date: None,
                all_time_low_change_percentage: None,
            },
        };

        let market = info.as_market();

        assert_eq!(market.all_time_high_change_percentage, Some(-20.0));
        assert_eq!(market.all_time_low_change_percentage, Some(100.0));
    }
}
