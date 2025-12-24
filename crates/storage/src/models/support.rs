use diesel::prelude::*;
use primitives::SupportDevice;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::support)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SupportRow {
    pub support_id: String,
    pub device_id: i32,
    pub unread: i32,
}

impl SupportRow {
    pub fn as_primitive(&self) -> SupportDevice {
        SupportDevice {
            support_device_id: self.support_id.clone(),
            unread: self.unread,
        }
    }
}
