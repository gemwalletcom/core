use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::devices_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceSessionRow {
    pub id: i32,
    pub device_id: i32,
    pub wallet_id: i32,
    pub signature: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::devices_sessions)]
pub struct NewDeviceSessionRow {
    pub device_id: i32,
    pub wallet_id: i32,
    pub signature: String,
}
