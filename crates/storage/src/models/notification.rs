use crate::sql_types::NotificationType;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::NotificationData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NotificationRow {
    pub id: i32,
    pub wallet_id: i32,
    pub notification_type: NotificationType,
    pub is_read: bool,
    pub metadata: Option<serde_json::Value>,
    pub read_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl NotificationRow {
    pub fn as_primitive(&self, wallet_identifier: String) -> NotificationData {
        NotificationData {
            wallet_id: wallet_identifier,
            notification_type: self.notification_type.0,
            is_read: self.is_read,
            metadata: self.metadata.clone(),
            read_at: self.read_at.map(|dt| dt.and_utc()),
            created_at: self.created_at.and_utc(),
        }
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::notifications)]
pub struct NewNotificationRow {
    pub wallet_id: i32,
    pub notification_type: NotificationType,
    pub metadata: Option<serde_json::Value>,
}
