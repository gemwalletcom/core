use crate::{Asset, NotificationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationData {
    pub wallet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRewardsMetadata {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRewardsRedeemMetadata {
    pub transaction_id: String,
    pub points: i32,
    pub value: String,
}
