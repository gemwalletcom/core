use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{Device, Platform, PlatformStore};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceRow {
    pub id: i32,
    pub device_id: String,
    pub platform: String,
    pub platform_store: Option<String>,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
    pub os: Option<String>,
    pub model: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateDeviceRow {
    pub device_id: String,
    pub platform: String,
    pub platform_store: Option<String>,
    pub token: String,
    pub locale: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: bool,
    pub version: String,
    pub subscriptions_version: i32,
    pub os: Option<String>,
    pub model: Option<String>,
}

impl DeviceRow {
    pub fn as_primitive(&self) -> Device {
        let platform = Platform::new(self.platform.as_str()).unwrap();
        let platform_store = PlatformStore::from_str(self.platform_store.clone().unwrap_or_default().as_str()).ok();

        Device {
            id: self.device_id.clone(),
            platform,
            os: self.os.clone(),
            model: self.model.clone(),
            platform_store,
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

impl UpdateDeviceRow {
    pub fn from_primitive(device: Device) -> Self {
        Self {
            device_id: device.id,
            platform: device.platform.as_str().to_string(),
            os: device.os,
            model: device.model,
            platform_store: device.platform_store.map(|x| x.as_ref().to_string()),
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
