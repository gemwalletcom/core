use alloy_primitives::{Address, AddressError};
use std::str::FromStr;

pub fn ethereum_address_checksum(address: &str) -> Result<String, AddressError> {
    Ok(Address::from_str(address)?.to_checksum(None))
}

pub fn ethereum_address_from_topic(topic: &str) -> Option<String> {
    ethereum_address_checksum(topic.trim_start_matches("0x000000000000000000000000")).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_address_checksum() {
        let expected = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a";
        assert_eq!(ethereum_address_checksum("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a").unwrap(), expected);
        assert_eq!(ethereum_address_checksum("5615e8ab93b9d695b6d4d6545f7792aa59e1069a").unwrap(), expected);
        assert!(ethereum_address_checksum("0X5615e8ab93b9d695b6d4d6545f7792aa59e1069a").is_err());
        assert!(ethereum_address_checksum("invalid").is_err());
    }

    #[test]
    fn test_ethereum_address_from_topic() {
        assert_eq!(
            ethereum_address_from_topic("0x0000000000000000000000005615e8ab93b9d695b6d4d6545f7792aa59e1069a"),
            Some("0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a".to_string())
        );
    }
}
