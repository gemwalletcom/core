use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddress {
    pub chain: String,
    pub name: Option<String>,
    pub address: String,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
}
