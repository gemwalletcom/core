use diesel::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::chains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChainRow {
    pub id: String,
}
