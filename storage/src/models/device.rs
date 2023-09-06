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
    pub is_push_enabled: bool,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDevice {
    pub device_id: String,
    pub platform: String,
    pub token: String,  
    pub locale: String,
    pub is_push_enabled: bool,
}

impl Device {
    pub fn as_primitive(&self) -> primitives::Device {
        let platform = primitives::Platform::from_str(self.platform.as_str()).unwrap();
        primitives::Device {
            id: self.device_id.clone(),
            platform,
            token: self.token.clone(),
            locale: self.locale.clone(),
            is_push_enabled: self.is_push_enabled,
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
            is_push_enabled: device.is_push_enabled,
        }
    }
}