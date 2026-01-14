use diesel::prelude::*;
use primitives::{AddressName, Chain, ScanAddress};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::sql_types::AddressType;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddressRow {
    pub id: i32,
    pub chain: String,
    pub address: String,
    pub name: Option<String>,
    #[diesel(column_name = type_)]
    pub type_: AddressType,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

impl ScanAddressRow {
    pub fn as_primitive(self) -> Option<AddressName> {
        self.name.map(|name| AddressName {
            chain: Chain::from_str(&self.chain).unwrap(),
            address: self.address,
            name,
        })
    }

    pub fn as_scan_address_primitive(self) -> ScanAddress {
        ScanAddress {
            chain: Chain::from_str(&self.chain).unwrap(),
            address: self.address,
            name: self.name,
            address_type: Some(self.type_.0.clone()),
            is_malicious: Some(self.is_fraudulent),
            is_memo_required: Some(self.is_memo_required),
            is_verified: Some(self.is_verified),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewScanAddressRow {
    pub chain: String,
    pub address: String,
    pub name: Option<String>,
    #[diesel(column_name = type_)]
    pub type_: AddressType,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
}

impl NewScanAddressRow {
    pub fn from_primitive(scan_address: ScanAddress) -> Self {
        Self {
            chain: scan_address.chain.as_ref().to_string(),
            address: scan_address.address,
            name: scan_address.name,
            type_: scan_address.address_type.unwrap_or(primitives::AddressType::Address).into(),
            is_verified: scan_address.is_verified.unwrap_or(false),
            is_fraudulent: scan_address.is_malicious.unwrap_or(false),
            is_memo_required: scan_address.is_memo_required.unwrap_or(false),
        }
    }
}
