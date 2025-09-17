use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::support)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Support {
    pub support_id: String,
    pub device_id: i32,
}
