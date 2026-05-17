use std::fmt;

use primitives::{Address, SignerError};

const POLKADOT_PREFIX: u8 = 0;
const ADDRESS_DATA_LENGTH: usize = 32;
const ADDRESS_CHECKSUM_LENGTH: usize = 2;
const ADDRESS_LENGTH: usize = 1 + ADDRESS_DATA_LENGTH + ADDRESS_CHECKSUM_LENGTH;
const ADDRESS_MAX_STRING_LENGTH: usize = 60;
const SS58_CHECKSUM_PREFIX: &[u8] = b"SS58PRE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolkadotAddress([u8; ADDRESS_DATA_LENGTH]);

impl Address for PolkadotAddress {
    fn try_parse(value: &str) -> Option<Self> {
        if value.len() > ADDRESS_MAX_STRING_LENGTH {
            return None;
        }

        let decoded = bs58::decode(value).into_vec().ok()?;
        if decoded.len() != ADDRESS_LENGTH || decoded[0] != POLKADOT_PREFIX {
            return None;
        }

        let checksum = Self::checksum(&decoded[..1 + ADDRESS_DATA_LENGTH]);
        if decoded[1 + ADDRESS_DATA_LENGTH..] != checksum {
            return None;
        }

        Some(Self(decoded[1..1 + ADDRESS_DATA_LENGTH].try_into().ok()?))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(ADDRESS_LENGTH);
        raw.push(POLKADOT_PREFIX);
        raw.extend_from_slice(&self.0);
        raw.extend_from_slice(&Self::checksum(&raw));
        bs58::encode(raw).into_string()
    }
}

impl PolkadotAddress {
    pub(crate) fn parse(value: &str) -> Result<Self, SignerError> {
        Self::try_parse(value).ok_or_else(|| SignerError::invalid_input("invalid Polkadot address"))
    }

    pub(crate) fn account_id(&self) -> &[u8; ADDRESS_DATA_LENGTH] {
        &self.0
    }

    fn checksum(data: &[u8]) -> [u8; ADDRESS_CHECKSUM_LENGTH] {
        let mut prefixed = Vec::with_capacity(SS58_CHECKSUM_PREFIX.len() + data.len());
        prefixed.extend_from_slice(SS58_CHECKSUM_PREFIX);
        prefixed.extend_from_slice(data);

        let hash = gem_hash::blake2::blake2b_512(&prefixed);
        let mut checksum = [0u8; ADDRESS_CHECKSUM_LENGTH];
        checksum.copy_from_slice(&hash[..ADDRESS_CHECKSUM_LENGTH]);
        checksum
    }
}

pub fn validate_address(address: &str) -> bool {
    PolkadotAddress::is_valid(address)
}

impl fmt::Display for PolkadotAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ADDRESS: &str = "15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo";
    const PUBLIC_KEY: &str = "cd3cfbbaa8f217c2a29ceae4b4063b597b629861916bad98f9826e03d1ab120e";

    #[test]
    fn test_polkadot_address() {
        let parsed = PolkadotAddress::from_str(VALID_ADDRESS).unwrap();

        assert!(validate_address(VALID_ADDRESS));
        assert_eq!(hex::encode(parsed.as_bytes()), PUBLIC_KEY);
        assert_eq!(parsed.to_string(), VALID_ADDRESS);
        assert_eq!(PolkadotAddress::try_parse(&parsed.encode()), Some(parsed));

        assert!(PolkadotAddress::from_str("invalid").is_err());
        assert!(!validate_address(&"1".repeat(ADDRESS_MAX_STRING_LENGTH + 1)));
        assert!(!validate_address("15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXj"));
    }
}
