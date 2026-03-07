use crate::sql_types::PushNotificationType;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::devices_notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceNotificationRow {
    pub id: i32,
    pub device_id: i32,
    pub notification_type: PushNotificationType,
    pub error: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::devices_notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDeviceNotificationRow {
    pub device_id: i32,
    pub notification_type: PushNotificationType,
    pub error: Option<String>,
}
