use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::Transaction;

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PushNotificationTypes {
    Test,        // Test payload
    Transaction, // PushNotificationTransaction (Transaction)
    PriceAlert,  // PriceAlert payload
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Option<serde_json::Value>,
}

// Only used to decode notification type
#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotificationPayloadType {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
}

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushNotificationTransaction {
    pub wallet_index: i32,
    pub transaction: Transaction,
}
