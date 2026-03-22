use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::PriceAlert;
use serde::{Deserialize, Serialize};

use crate::sql_types::{AssetId, PriceAlertDirectionRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAlertRow {
    pub identifier: String,
    pub device_id: i32,
    pub asset_id: AssetId,
    pub currency: String,
    pub price_direction: Option<PriceAlertDirectionRow>,
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
    pub asset_id: AssetId,
    pub currency: String,
    pub price_direction: Option<PriceAlertDirectionRow>,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
}

impl PriceAlertRow {
    pub fn as_primitive(&self) -> PriceAlert {
        PriceAlert {
            asset_id: self.asset_id.0.clone(),
            currency: self.currency.clone(),
            price_direction: self.price_direction.as_ref().map(|value| value.0.clone()),
            price: self.price,
            price_percent_change: self.price_percent_change,
            last_notified_at: self.last_notified_at.map(|x| x.and_utc()),
            identifier: self.identifier.clone(),
        }
    }

    pub fn new_price_alert(primitive: PriceAlert, device_id: i32) -> NewPriceAlertRow {
        NewPriceAlertRow {
            identifier: primitive.id(),
            device_id,
            asset_id: primitive.asset_id.into(),
            currency: primitive.currency.clone(),
            price_direction: primitive.price_direction.map(Into::into),
            price: primitive.price,
            price_percent_change: primitive.price_percent_change,
        }
    }
}
