use hex::FromHex;
use primitives::SignerError;
use serde::{Deserialize, Serialize};

const ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub fn from_hex(value: &str) -> Result<Self, SignerError> {
        let trimmed = value.trim();
        let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
        if hex_str.is_empty() {
            return Err(SignerError::InvalidInput("Empty Aptos address".to_string()));
        }
        let padded = if hex_str.len() % 2 == 1 {
            format!("0{}", hex_str)
        } else {
            hex_str.to_string()
        };
        let mut bytes = Vec::from_hex(padded.as_bytes()).map_err(|_| SignerError::InvalidInput("Invalid Aptos address hex".to_string()))?;
        if bytes.len() > ADDRESS_LENGTH {
            return Err(SignerError::InvalidInput("Aptos address too long".to_string()));
        }
        if bytes.len() < ADDRESS_LENGTH {
            let mut padded_bytes = vec![0u8; ADDRESS_LENGTH - bytes.len()];
            padded_bytes.append(&mut bytes);
            bytes = padded_bytes;
        }
        let mut address = [0u8; ADDRESS_LENGTH];
        address.copy_from_slice(&bytes);
        Ok(Self(address))
    }

    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    pub fn one() -> Self {
        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes[ADDRESS_LENGTH - 1] = 1;
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_short_hex() {
        let address = AccountAddress::from_hex("0x1").unwrap();
        assert_eq!(address.to_hex(), format!("0x{}", "00".repeat(31) + "01"));
    }
}
