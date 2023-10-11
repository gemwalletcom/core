use typeshare::typeshare;
use serde::{Serialize, Deserialize};
use crate::Transaction;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PushNotificationTypes {
    Transaction,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct PushNotification {
    #[serde(rename = "type")]
    pub notification_type: PushNotificationTypes,
    pub data: Transaction,
}