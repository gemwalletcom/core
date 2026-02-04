use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::devices_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceSessionRow {
    pub id: i32,
    pub device_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::devices_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDeviceSessionRow {
    pub device_id: i32,
}
