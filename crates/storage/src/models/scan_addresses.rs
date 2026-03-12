use diesel::prelude::*;
use primitives::{AddressName, ScanAddress, VerificationStatus};
use serde::{Deserialize, Serialize};

use crate::sql_types::{AddressType, ChainRow};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::scan_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ScanAddressRow {
    pub id: i32,
    pub chain: ChainRow,
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
        Some(AddressName {
            chain: self.chain.0,
            address: self.address,
            name: self.name?,
            address_type: Some(self.type_.0.clone()),
            status: if self.is_fraudulent {
                VerificationStatus::Suspicious
            } else if self.is_verified {
                VerificationStatus::Verified
            } else {
                VerificationStatus::Unverified
            },
        })
    }

    pub fn as_scan_address_primitive(self) -> ScanAddress {
        ScanAddress {
            chain: self.chain.0,
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
    pub chain: ChainRow,
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
            chain: ChainRow::from(scan_address.chain),
            address: scan_address.address,
            name: scan_address.name,
            type_: scan_address.address_type.unwrap_or(primitives::AddressType::Address).into(),
            is_verified: scan_address.is_verified.unwrap_or(false),
            is_fraudulent: scan_address.is_malicious.unwrap_or(false),
            is_memo_required: scan_address.is_memo_required.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScanAddressRow;
    use crate::sql_types::{AddressType, ChainRow};
    use chrono::DateTime;
    use primitives::{AddressType as PrimitiveAddressType, Chain};

    #[test]
    fn as_primitive_returns_none_without_name() {
        let row = ScanAddressRow {
            id: 1,
            chain: ChainRow::from(Chain::Ethereum),
            address: "0x0000000000000000000000000000000000000001".to_string(),
            name: None,
            type_: AddressType::from(PrimitiveAddressType::Address),
            is_verified: false,
            is_fraudulent: false,
            is_memo_required: false,
            updated_at: DateTime::UNIX_EPOCH.naive_utc(),
            created_at: DateTime::UNIX_EPOCH.naive_utc(),
        };

        assert_eq!(row.as_primitive(), None);
    }
}
