use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId, PriceAlert};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAlertRow {
    pub identifier: String,
    pub device_id: i32,
    pub asset_id: String,
    pub currency: String,
    pub price_direction: Option<String>,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
    pub last_notified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPriceAlertRow {
    pub identifier: String,
    pub device_id: i32,
    pub asset_id: String,
    pub currency: String,
    pub price_direction: Option<String>,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
}

impl PriceAlertRow {
    pub fn as_primitive(&self) -> PriceAlert {
        PriceAlert {
            asset_id: AssetId::new(&self.asset_id).unwrap(),
            currency: self.currency.clone(),
            price_direction: self.price_direction.as_deref().and_then(|value| value.parse().ok()),
            price: self.price,
            price_percent_change: self.price_percent_change,
            last_notified_at: self.last_notified_at.map(|x| x.and_utc()),
        }
    }

    pub fn new_price_alert(primitive: PriceAlert, device_id: i32) -> NewPriceAlertRow {
        NewPriceAlertRow {
            identifier: primitive.id(),
            device_id,
            asset_id: primitive.asset_id.to_string(),
            currency: primitive.currency.clone(),
            price_direction: primitive.price_direction.map(|value| value.as_ref().to_string()),
            price: primitive.price,
            price_percent_change: primitive.price_percent_change,
        }
    }
}
