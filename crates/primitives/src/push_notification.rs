use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Device, Platform};

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PushNotificationTypes {
    Test,        // Test payload
    Transaction, // PushNotificationTransaction (Transaction)
    Asset,
    PriceAlert, // PriceAlert payload
    BuyAsset,   // PushNotificationBuyAsset payload
    SwapAsset,  // PushNotificationSwapAsset payload
    Support,    // PushNotificationSupport payload
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotifications {
    pub notifications: Vec<GorushNotification>,
}

// Notification used to send to Pusher
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: Option<String>,
    pub data: PushNotification,
}

impl GorushNotification {
    pub fn new(tokens: Vec<String>, platform: Platform, title: String, message: String, data: PushNotification) -> Self {
        Self {
            tokens,
            platform: platform.as_i32(),
            title,
            message,
            topic: None,
            data,
        }
    }

    pub fn from_device(device: Device, title: String, message: String, data: PushNotification) -> Self {
        Self {
            tokens: vec![device.token],
            platform: device.platform.as_i32(),
            title,
            message,
            topic: None,
            data,
        }
    }

    pub fn with_topic(mut self, topic: Option<String>) -> Self {
        self.topic = topic;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Option<serde_json::Value>,
}

impl PushNotification {
    pub fn new_buy_asset(asset_id: AssetId) -> Self {
        Self {
            notification_type: PushNotificationTypes::BuyAsset,
            data: serde_json::to_value(PushNotificationAsset {
                asset_id: asset_id.to_string(),
            })
            .ok(),
        }
    }
}

// Only used to decode notification type
#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotificationPayloadType {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationTransaction {
    pub wallet_index: i32,
    pub asset_id: String,
    pub transaction_id: String,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationAsset {
    pub asset_id: String,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationSwapAsset {
    pub from_asset_id: String,
    pub to_asset_id: String,
}
