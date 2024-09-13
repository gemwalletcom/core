use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub platform: Platform,
    pub token: String,
    pub locale: String,
    pub version: String,
    pub currency: String,
    pub is_push_enabled: bool,
    pub is_price_alerts_enabled: Option<bool>,
    pub subscriptions_version: i32,
}
