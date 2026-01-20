use crate::NotificationType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub wallet_id: String,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct NotificationRewardsMetadata {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i32>,
}
