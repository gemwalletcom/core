use diesel::prelude::*;
use primitives::LinkType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::link_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LinkTypeRow {
    pub id: String,
    pub name: String,
}

impl LinkTypeRow {
    pub fn from_primitive(primitive: LinkType) -> Self {
        Self {
            id: primitive.as_ref().to_string(),
            name: primitive.name(),
        }
    }
}
