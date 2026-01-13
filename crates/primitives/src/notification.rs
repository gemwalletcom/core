use crate::NotificationType;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub wallet_id: String,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}
