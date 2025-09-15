use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::support)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Support {
    pub support_id: String,
    pub device_id: i32,
}

impl Support {
    pub fn as_primitive(&self) -> primitives::SupportDevice {
        primitives::SupportDevice {
            support_id: self.support_id.clone(),
            device_id: self.device_id.to_string(),
        }
    }
}
