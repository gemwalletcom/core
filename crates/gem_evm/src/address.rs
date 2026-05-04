use alloy_primitives::{Address, AddressError};
use primitives::Address as AddressTrait;
use std::str::FromStr;

pub fn ethereum_address_checksum(address: &str) -> Result<String, AddressError> {
    Ok(Address::from_str(address)?.to_checksum(None))
}

pub fn ethereum_address_from_topic(topic: &str) -> Option<String> {
    ethereum_address_checksum(topic.trim_start_matches("0x000000000000000000000000")).ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthereumAddress(Address);

impl AddressTrait for EthereumAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Address::from_str(address).ok().map(Self)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn encode(&self) -> String {
        self.0.to_checksum(None)
    }
}

pub fn validate_address(address: &str) -> bool {
    EthereumAddress::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const VALID_ADDRESS: &str = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a";

    #[test]
    fn test_ethereum_address() {
        let lowercase = VALID_ADDRESS.to_lowercase();
        let uppercase_prefix = lowercase.replacen("0x", "0X", 1);

        assert_eq!(ethereum_address_checksum(&lowercase).unwrap(), VALID_ADDRESS);
        assert_eq!(ethereum_address_checksum(lowercase.trim_start_matches("0x")).unwrap(), VALID_ADDRESS);
        assert!(ethereum_address_checksum(&uppercase_prefix).is_err());
        assert!(ethereum_address_checksum("invalid").is_err());

        let parsed = EthereumAddress::try_parse(&lowercase).unwrap();
        assert!(validate_address(&lowercase));
        assert_eq!(parsed.as_bytes().len(), 20);
        assert_eq!(parsed.encode(), VALID_ADDRESS);
        assert!(!validate_address(&uppercase_prefix));
    }

    #[test]
    fn test_ethereum_address_from_topic() {
        assert_eq!(
            ethereum_address_from_topic("0x0000000000000000000000005615e8ab93b9d695b6d4d6545f7792aa59e1069a"),
            Some(VALID_ADDRESS.to_string())
        );
    }
}
