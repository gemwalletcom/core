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
        AssetMarket {
            market_cap: self.market.market_cap.map(|x| x * rate),
            market_cap_fdv: self.market.market_cap_fdv.map(|x| x * rate),
            market_cap_rank: self.market.market_cap_rank,
            total_volume: self.market.total_volume.map(|x| x * rate),
            circulating_supply: self.market.circulating_supply,
            total_supply: self.market.total_supply,
            max_supply: self.market.max_supply,
        }
    }
}
