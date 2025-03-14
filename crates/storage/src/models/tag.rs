use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Tag {
    pub id: String,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::assets_tags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssetTag {
    pub asset_id: String,
    pub tag_id: String,
    pub order: Option<i32>,
}

impl Tag {
    pub fn from_primitive(primitive: primitives::AssetTag) -> Self {
        Self {
            id: primitive.as_ref().to_lowercase(),
        }
    }
}
