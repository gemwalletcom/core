use alloy_primitives::{Address, AddressError};
use std::str::FromStr;

pub fn ethereum_address_checksum(address: &str) -> Result<String, AddressError> {
    let address = Address::from_str(address)?;
    Ok(address.to_checksum(None))
}

pub fn ethereum_address_from_topic(topic: &str) -> Option<String> {
    ethereum_address_checksum(topic.trim_start_matches("0x000000000000000000000000")).ok()
}
