use diesel::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetType {
    pub id: String,
}
