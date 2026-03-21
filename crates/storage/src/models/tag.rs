use diesel::prelude::*;
use primitives::AssetTag;
use serde::{Deserialize, Serialize};

use crate::sql_types::AssetId;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TagRow {
    pub id: String,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetTagRow {
    pub asset_id: AssetId,
    pub tag_id: String,
    pub order: Option<i32>,
}

impl TagRow {
    pub fn from_primitive(primitive: AssetTag) -> Self {
        Self {
            id: primitive.as_ref().to_lowercase(),
        }
    }
}
