use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::tokenlists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TokenList {
    pub id: i32,
    pub chain: String,
    pub url: String,
    pub version: i32,
}
