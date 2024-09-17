use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    pub id: i32,
    pub device_id: String,
    pub platform: String,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDevice {
    pub device_id: String,
    pub platform: String,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
}

impl Device {
    pub fn as_primitive(&self) -> primitives::Device {
        let platform = primitives::Platform::new(self.platform.as_str()).unwrap();
        primitives::Device {
            id: self.device_id.clone(),
            platform,
            token: self.token.clone(),
            locale: self.locale.clone(),
            currency: self.currency.clone(),
            is_push_enabled: self.is_push_enabled,
            is_price_alerts_enabled: Some(self.is_price_alerts_enabled),
            version: self.version.clone(),
            subscriptions_version: self.subscriptions_version,
        }
    }
}

impl UpdateDevice {
    pub fn from_primitive(device: primitives::Device) -> Self {
        Self {
            device_id: device.id,
            platform: device.platform.as_str().to_string(),
            token: device.token,
            locale: device.locale,
            currency: device.currency,
            is_push_enabled: device.is_push_enabled,
            is_price_alerts_enabled: device.is_price_alerts_enabled.unwrap_or(false),
            version: device.version,
            subscriptions_version: device.subscriptions_version,
        }
    }
}
