use primitives::{Address as AddressTrait, SignerError, decode_hex};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub fn from_hex(value: &str) -> Result<Self, SignerError> {
        <Self as FromStr>::from_str(value)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignerError> {
        if bytes.len() > ADDRESS_LENGTH {
            return Err(SignerError::InvalidInput("Aptos address too long".to_string()));
        }
        let mut address = [0u8; ADDRESS_LENGTH];
        let offset = ADDRESS_LENGTH - bytes.len();
        address[offset..].copy_from_slice(bytes);
        Ok(Self(address))
    }

    pub fn one() -> Self {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes[ADDRESS_LENGTH - 1] = 1;
        Self(bytes)
    }
}

impl FromStr for AccountAddress {
    type Err = SignerError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = decode_hex(value)?;
        Self::from_bytes(&bytes)
    }
}

impl AddressTrait for AccountAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Self::from_hex(address).ok()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        self.to_string()
    }
}

pub fn validate_address(address: &str) -> bool {
    AccountAddress::is_valid(address)
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", ::hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ADDRESS: &str = "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc";

    #[test]
    fn test_aptos_address() {
        let parsed = AccountAddress::from_hex(VALID_ADDRESS).unwrap();

        assert!(validate_address(VALID_ADDRESS));
        assert_eq!(parsed.to_string(), VALID_ADDRESS);
        assert_eq!(parsed.as_bytes().len(), 32);
        assert!(!validate_address("invalid"));

        // short hex is left-padded to 32 bytes (Aptos framework address convention)
        let short = AccountAddress::from_hex("0x1").unwrap();
        assert_eq!(short.to_string(), format!("0x{}", "00".repeat(31) + "01"));
    }
}
