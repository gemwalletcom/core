use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::parser_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ParserState {
    pub chain: String,
    pub current_block: i32,
    pub latest_block: i32,
    pub await_blocks: i32,
    pub is_enabled: bool,
}
