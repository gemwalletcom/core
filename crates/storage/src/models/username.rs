use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, Clone)]
#[diesel(table_name = crate::schema::usernames)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UsernameRow {
    pub username: String,
    pub address: String,
    pub is_verified: bool,
    pub is_rewards_enabled: bool,
    pub rewards_level: Option<String>,
}
