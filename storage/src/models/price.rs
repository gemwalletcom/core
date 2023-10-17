use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetPrice, AssetMarket};
use serde::{Deserialize, Serialize};
use std::{hash::{Hash, Hasher}, time::{UNIX_EPOCH, SystemTime}};

use super::chart::Chart;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Price {
    pub asset_id: String,
    pub coin_id: String,
    pub price: f64,
    pub price_change_percentage_24h: f64,
    pub market_cap: f64,
    pub market_cap_rank: i32,
    pub total_volume: f64,
    pub last_updated_at: NaiveDateTime,
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}
impl Eq for Price {}

impl Hash for Price {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.asset_id.hash(state);
    }
}

impl Price {
    pub fn new(
        asset_id: String, 
        coin_id: String,
        price: f64, 
        price_change_percentage_24h: f64,
        market_cap: f64,
        market_cap_rank: i32,
        total_volume: f64,
    ) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        let last_updated_at = NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, 0).unwrap();

        Price {
            asset_id,
            coin_id,
            price,
            price_change_percentage_24h,
            last_updated_at,
            market_cap,
            market_cap_rank,
            total_volume,
        }
    }
}

impl Price {
    pub fn chart_value(&self) -> Chart {
        Chart {
            coin_id: self.coin_id.clone(),
            price: self.price,
            date: self.last_updated_at,
            market_cap: self.market_cap,
            volume: self.total_volume,
        }
    }

    pub fn as_primitive(&self) -> AssetPrice {
        AssetPrice{
            asset_id: self.asset_id.clone(),
            price: self.price,
            price_change_percentage_24h: self.price_change_percentage_24h,
        }
    }

    pub fn as_price_primitive(&self) -> primitives::Price {
        primitives::Price{
            price: self.price,
            price_change_percentage_24h: self.price_change_percentage_24h,
        }
    }

    pub fn as_market_primitive(&self) -> AssetMarket {
        AssetMarket{
            market_cap: Some(self.market_cap).into(),
            market_cap_rank: Some(self.market_cap_rank),
            total_volume: self.total_volume.into(),
        }
    }
}