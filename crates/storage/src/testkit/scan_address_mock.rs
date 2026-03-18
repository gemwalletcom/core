use crate::models::ScanAddressRow;
use crate::sql_types::{AddressType, ChainRow};
use chrono::DateTime;
use primitives::{AddressType as PrimitiveAddressType, Chain};

impl ScanAddressRow {
    pub fn mock(id: i32, chain: Chain, address: &str, name: Option<&str>) -> Self {
        Self {
            id,
            chain: ChainRow::from(chain),
            address: address.to_string(),
            name: name.map(str::to_string),
            type_: AddressType::from(PrimitiveAddressType::Contract),
            is_verified: false,
            is_fraudulent: false,
            is_memo_required: false,
            updated_at: DateTime::UNIX_EPOCH.naive_utc(),
            created_at: DateTime::UNIX_EPOCH.naive_utc(),
        }
    }
}
