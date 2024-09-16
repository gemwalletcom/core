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
    pub price: Option<f64>,
    pub last_notified_at: Option<NaiveDateTime>,
}

impl PriceAlert {
    pub fn as_primitive(&self) -> primitives::PriceAlert {
        primitives::PriceAlert {
            asset_id: self.asset_id.clone(),
            price: self.price,
        }
    }

    pub fn from_primitive(primitive: primitives::PriceAlert, device_id: i32) -> Self {
        Self {
            id: 0,
            device_id,
            asset_id: primitive.asset_id,
            price: primitive.price,
            last_notified_at: None,
        }
    }
}
