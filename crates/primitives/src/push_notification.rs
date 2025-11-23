use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetId, Transaction};

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
    #[typeshare(skip)]
    pub transaction_id: String,
    pub transaction: Transaction,
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

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationSupport {}
