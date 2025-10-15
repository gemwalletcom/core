use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_dex_providers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceDexProvider {
    pub id: String,
    pub enabled: bool,
    pub priority: i32,
}

impl PriceDexProvider {
    pub fn new(id: String, priority: i32) -> Self {
        Self { id, enabled: true, priority }
    }
}

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_dex)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceDex {
    pub id: String,
    pub provider: String,
    pub price: f64,
    pub last_updated_at: NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::prices_dex_assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceDexAsset {
    pub asset_id: String,
    pub price_feed_id: String,
}

impl PriceDex {
    pub fn new(id: String, provider: String, price: f64, last_updated_at: NaiveDateTime) -> Self {
        Self {
            id,
            provider,
            price,
            last_updated_at,
        }
    }
}

impl PriceDexAsset {
    pub fn new(asset_id: String, price_feed_id: String) -> Self {
        Self { asset_id, price_feed_id }
    }
}
