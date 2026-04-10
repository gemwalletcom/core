use gem_encoding::{decode_base32, encode_base32};
use primitives::Address;
use signer::Base32Address;
use std::fmt;

const ED25519_PUBLIC_KEY_VERSION: u8 = 0x30;
const ADDRESS_LENGTH: usize = 56;
const DECODED_LENGTH: usize = 35;
const CRC16_XMODEM_POLY: u16 = 0x1021;

#[derive(Clone)]
pub struct StellarAddress {
    pub(crate) base32: Base32Address,
}

impl Address for StellarAddress {
    fn try_parse(address: &str) -> Option<Self> {
        if address.len() != ADDRESS_LENGTH || !address.starts_with('G') {
            return None;
        }
        let decoded = decode_base32(address.as_bytes()).ok()?;
        if decoded.len() != DECODED_LENGTH || decoded[0] != ED25519_PUBLIC_KEY_VERSION {
            return None;
        }
        let crc = u16::from_le_bytes([decoded[33], decoded[34]]);
        if Self::crc16_xmodem(&decoded[..33]) != crc {
            return None;
        }
        Base32Address::from_slice(&decoded[1..33]).ok().map(|base32| Self { base32 })
    }

    fn as_bytes(&self) -> &[u8] {
        self.base32.payload()
    }

    fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(DECODED_LENGTH);
        raw.push(ED25519_PUBLIC_KEY_VERSION);
        raw.extend_from_slice(self.base32.payload());
        let crc = Self::crc16_xmodem(&raw).to_le_bytes();
        raw.extend_from_slice(&crc);
        encode_base32(&raw)
    }
}

impl StellarAddress {
    /// Last 4 bytes of the public key, used as a key hint in Stellar signature envelopes.
    pub(crate) fn key_hint(&self) -> &[u8; 4] {
        self.base32.payload()[28..32].try_into().unwrap()
    }

    fn crc16_xmodem(data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                crc = if crc & 0x8000 != 0 { (crc << 1) ^ CRC16_XMODEM_POLY } else { crc << 1 };
            }
        }
        crc
    }
}

impl fmt::Display for StellarAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_address() {
        assert!(StellarAddress::is_valid("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI"));
        assert!(!StellarAddress::is_valid(""));
        assert!(!StellarAddress::is_valid("invalid"));
        assert!(!StellarAddress::is_valid("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYZ"));

        let addr = StellarAddress::from_str("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI").unwrap();
        assert_eq!(addr.to_string(), "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI");
        assert_eq!(addr.as_bytes().len(), 32);
    }
}
