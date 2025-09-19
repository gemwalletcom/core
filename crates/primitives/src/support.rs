use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SupportDevice {
    pub support_id: String,
    #[serde(skip_serializing)]
    pub device_id: String,
    pub unread: i32,
}
