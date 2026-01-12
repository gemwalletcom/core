use crate::sql_types::UsernameStatus;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::usernames)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UsernameRow {
    pub username: String,
    pub address: String,
    pub status: UsernameStatus,
}

impl UsernameRow {
    pub fn has_custom_username(&self) -> bool {
        !self.username.eq_ignore_ascii_case(&self.address)
    }

    pub fn is_verified(&self) -> bool {
        self.status.is_verified()
    }
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = crate::schema::usernames)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUsernameRow {
    pub username: String,
    pub address: String,
    pub status: UsernameStatus,
}
