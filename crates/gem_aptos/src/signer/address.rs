use primitives::SignerError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

const ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub fn from_str(value: &str) -> Result<Self, SignerError> {
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
        let stripped = primitives::strip_0x(value);
        if stripped.is_empty() {
            return Err(SignerError::InvalidInput("Empty Aptos address".to_string()));
        }
        let bytes = primitives::decode_hex(value)
            .map_err(|_| SignerError::InvalidInput("Invalid Aptos address hex".to_string()))?;
        Self::from_bytes(&bytes)
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", ::hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_short_hex() {
        let address = AccountAddress::from_str("0x1").unwrap();
        assert_eq!(address.to_string(), format!("0x{}", "00".repeat(31) + "01"));
    }
}
