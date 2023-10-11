use typeshare::typeshare;
use serde::{Serialize, Deserialize};
use crate::Transaction;

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PushNotificationTypes {
    Transaction,
}

#[typeshare(swift = "Equatable, Codable")]
#[derive(Debug, Serialize, Deserialize)]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Transaction,
}