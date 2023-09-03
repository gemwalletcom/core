use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::{hash::{Hash, Hasher}, time::{UNIX_EPOCH, SystemTime}};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::nodes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Node {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub status: String,
    pub priority: i32,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::tokenlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenList {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub version: i32,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::versions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Version {
    pub id: i32,
    pub platform: String,
    pub production: String,
    pub beta: String,
    pub alpha: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::fiat_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatAsset {
    pub id: i32,
    pub asset: String,
    pub provider: String,
    pub symbol: String,
    pub network: Option<String>,
}

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

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::fiat_rates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FiatRate {
    pub symbol: String,
    pub name: String,
    pub rate: f64,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chart {
    pub coin_id: String,
    pub price: f64,
    pub date: NaiveDateTime,
    pub market_cap: f64,
    pub volume: f64,
}

impl PartialEq for Chart {
    fn eq(&self, other: &Self) -> bool {
        self.coin_id == other.coin_id && self.date == other.date
    }
}
impl Eq for Chart {}

impl Hash for Chart {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.coin_id.hash(state);
        self.date.hash(state);
    }
}

pub type ChartResult = (chrono::NaiveDateTime, f64);

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
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    pub device_id: String,
    pub platform: i32,
    pub token: String,  
    pub is_push_enabled: bool,
}