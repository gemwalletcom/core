use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddress {
    pub name: Option<String>,
    pub address: String,
    pub is_verified: bool,
    pub is_fradulent: bool,
    pub is_memo_required: bool,
}

impl ScanAddress {
    pub fn as_primitive(&self) -> primitives::ScanAddress {
        primitives::ScanAddress {
            name: self.name.clone(),
            address: self.address.clone(),
            is_verified: self.is_verified,
            is_fradulent: self.is_fradulent,
            is_memo_required: self.is_memo_required,
        }
    }
}

