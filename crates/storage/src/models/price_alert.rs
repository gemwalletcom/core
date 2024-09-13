use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::PriceAlertSubsription;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::price_alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PriceAlert {
    pub device_id: i32,
    pub asset_id: String,
    pub last_notified_at: Option<NaiveDateTime>,
}

impl PriceAlert {
    pub fn as_primitive(&self) -> PriceAlertSubsription {
        primitives::PriceAlertSubsription {
            asset_id: self.asset_id.clone(),
        }
    }

    pub fn from_primitive(primitive: PriceAlertSubsription, device_id: i32) -> Self {
        Self {
            device_id,
            asset_id: primitive.asset_id,
            last_notified_at: None,
        }
    }
}
