use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::platform::Platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift="Codable")]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub platform: Platform,
    pub token: String,
    pub is_push_enabled: bool,
}