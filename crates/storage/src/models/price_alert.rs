use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAlert {
    pub id: i32,
    pub device_id: i32,
    pub asset_id: String,
    pub price_direction: Option<String>,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
    pub last_notified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPriceAlert {
    pub device_id: i32,
    pub asset_id: String,
    pub price_direction: Option<String>,
    pub price: Option<f64>,
    pub price_percent_change: Option<f64>,
}

impl PriceAlert {
    pub fn as_primitive(&self) -> primitives::PriceAlert {
        primitives::PriceAlert {
            asset_id: self.asset_id.clone(),
            price_direction: self.price_direction.as_deref().and_then(|value| value.parse().ok()),
            price: self.price,
            price_percent_change: self.price_percent_change,
        }
    }

    pub fn new_price_alert(primitive: primitives::PriceAlert, device_id: i32) -> NewPriceAlert {
        NewPriceAlert {
            device_id,
            asset_id: primitive.asset_id.clone(),
            price_direction: primitive.price_direction.map(|value| value.as_ref().to_string()),
            price: primitive.price,
            price_percent_change: primitive.price_percent_change,
        }
    }
}
