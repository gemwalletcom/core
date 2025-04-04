use alloy_primitives::{Address, AddressError};
use std::str::FromStr;

pub fn to_checksum(address: &Address) -> String {
    address.to_checksum(None)
}

pub fn normalize_checksum(address: &str) -> Result<String, AddressError> {
    let address = Address::from_str(address)?;
    Ok(address.to_checksum(None))
}
