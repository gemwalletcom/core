use crate::Transaction;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PushNotificationTypes {
    Transaction,
    PriceAlertClient,
}

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Transaction,
}
