use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::Device;
use serde::{Deserialize, Serialize};

use crate::sql_types::{Platform, PlatformStore};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceRow {
    pub id: i32,
    pub device_id: String,
    pub platform: Platform,
    pub platform_store: PlatformStore,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
    pub os: String,
    pub model: String,
    pub public_key: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDeviceRow {
    pub device_id: String,
    pub platform: Platform,
    pub platform_store: PlatformStore,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
    pub os: String,
    pub model: String,
    pub public_key: Option<String>,
}

impl DeviceRow {
    pub fn as_primitive(&self) -> Device {
        Device {
            id: self.device_id.clone(),
            platform: self.platform.0,
            platform_store: self.platform_store.0,
            os: self.os.clone(),
            model: self.model.clone(),
            token: self.token.clone(),
            locale: self.locale.clone(),
            currency: self.currency.clone(),
            is_push_enabled: self.is_push_enabled,
            is_price_alerts_enabled: Some(self.is_price_alerts_enabled),
            version: self.version.clone(),
            subscriptions_version: self.subscriptions_version,
            public_key: self.public_key.clone(),
        }
    }
}

impl UpdateDeviceRow {
    pub fn from_primitive(device: Device) -> Self {
        Self {
            device_id: device.id,
            platform: device.platform.into(),
            platform_store: device.platform_store.into(),
            os: device.os,
            model: device.model,
            token: device.token,
            locale: device.locale,
            currency: device.currency,
            is_push_enabled: device.is_push_enabled,
            is_price_alerts_enabled: device.is_price_alerts_enabled.unwrap_or(false),
            version: device.version,
            subscriptions_version: device.subscriptions_version,
            public_key: device.public_key,
        }
    }
}
