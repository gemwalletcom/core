use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::Platform;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SupportDevice {
    pub support_id: String,
    pub device_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
pub struct SupportCustomAttributes {
    pub platform: Platform,
    pub os: String,
    pub device: String,
    pub currency: String,
    pub app_version: String,
}
