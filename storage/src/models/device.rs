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
            currency: self.currency.clone().into(),
            is_push_enabled: self.is_push_enabled,
            version: self.version.clone().into(),
            subscriptions_version: self.subscriptions_version.into(),
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
            currency: device.currency.unwrap_or_default(),
            is_push_enabled: device.is_push_enabled,
            version: device.version.unwrap_or_default(),
            subscriptions_version: device.subscriptions_version.unwrap_or_default(),
        }
    }
}