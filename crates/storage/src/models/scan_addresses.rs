use diesel::prelude::*;
use primitives::{AddressName as PrimitivesAddressName, Chain};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl ScanAddress {
    pub fn as_primitive(self) -> Option<PrimitivesAddressName> {
        self.name.map(|name| PrimitivesAddressName {
            chain: Chain::from_str(&self.chain).unwrap(),
            address: self.address,
            name,
        })
    }
}

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddressType {
    pub id: String,
}

impl ScanAddressType {
    pub fn from_primitive(primitive: primitives::AddressType) -> Self {
        Self {
            id: primitive.as_ref().to_lowercase(),
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewScanAddress {
    pub chain: String,
    pub address: String,
    pub name: Option<String>,
    #[diesel(column_name = type_)]
    pub type_: Option<String>,
    pub is_verified: bool,
    pub is_fraudulent: bool,
    pub is_memo_required: bool,
}

impl NewScanAddress {
    pub fn from_primitive(scan_address: primitives::ScanAddress) -> Self {
        Self {
            chain: scan_address.chain.as_ref().to_string(),
            address: scan_address.address,
            name: scan_address.name,
            type_: scan_address.address_type.map(|t| t.as_ref().to_lowercase()),
            is_verified: scan_address.is_verified.unwrap_or(false),
            is_fraudulent: scan_address.is_malicious.unwrap_or(false),
            is_memo_required: scan_address.is_memo_required.unwrap_or(false),
        }
    }
}
