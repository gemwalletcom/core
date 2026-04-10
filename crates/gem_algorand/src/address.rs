use gem_encoding::{decode_base32, encode_base32};
use gem_hash::sha2::sha512_256;
use primitives::Address;
use signer::Base32Address;
use std::fmt;

const ADDRESS_DATA_LENGTH: usize = 32;
const ADDRESS_CHECKSUM_LENGTH: usize = 4;

#[derive(Clone)]
pub struct AlgorandAddress {
    pub(crate) base32: Base32Address,
}

impl Address for AlgorandAddress {
    fn try_parse(address: &str) -> Option<Self> {
        let decoded = decode_base32(address.as_bytes()).ok()?;
        if decoded.len() != ADDRESS_DATA_LENGTH + ADDRESS_CHECKSUM_LENGTH {
            return None;
        }
        let base32 = Base32Address::from_slice(&decoded[..ADDRESS_DATA_LENGTH]).ok()?;
        (decoded[ADDRESS_DATA_LENGTH..] == Self::checksum(base32.payload())).then_some(Self { base32 })
    }

    fn as_bytes(&self) -> &[u8] {
        self.base32.payload()
    }

    fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(ADDRESS_DATA_LENGTH + ADDRESS_CHECKSUM_LENGTH);
        raw.extend_from_slice(self.base32.payload());
        raw.extend_from_slice(&Self::checksum(self.base32.payload()));
        encode_base32(&raw)
    }
}

impl AlgorandAddress {
    fn checksum(bytes: &[u8; 32]) -> [u8; ADDRESS_CHECKSUM_LENGTH] {
        let digest = sha512_256(bytes);
        let mut checksum = [0u8; ADDRESS_CHECKSUM_LENGTH];
        checksum.copy_from_slice(&digest[digest.len() - ADDRESS_CHECKSUM_LENGTH..]);
        checksum
    }
}

impl fmt::Display for AlgorandAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorand_address() {
        assert!(AlgorandAddress::is_valid("QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQU"));
        assert!(!AlgorandAddress::is_valid(""));
        assert!(!AlgorandAddress::is_valid("invalid"));
        assert!(!AlgorandAddress::is_valid("QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQX"));

        let addr = AlgorandAddress::from_str("QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQU").unwrap();
        assert_eq!(addr.to_string(), "QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQU");
        assert_eq!(addr.as_bytes().len(), 32);
    }
}
