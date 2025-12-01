use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct NewSupportDevice {
    pub support_device_id: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SupportDevice {
    pub support_device_id: String,
    pub unread: i32,
}
