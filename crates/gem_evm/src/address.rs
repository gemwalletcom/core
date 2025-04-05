use alloy_primitives::{Address, AddressError};
use std::str::FromStr;

pub fn ethereum_address_checksum(address: &str) -> Result<String, AddressError> {
    let address = Address::from_str(address)?;
    Ok(address.to_checksum(None))
}
