use std::fmt;
use std::str::FromStr;

use crc::Crc;
use gem_encoding::{decode_base64_no_pad, decode_base64_url, encode_base64_url};
use primitives::{Address as AddressTrait, AddressError, SignerError};

type Workchain = i32;
type HashPart = [u8; 32];
type RawBytes = [u8; 33];

const USER_FRIENDLY_FLAG: u8 = 0x11;
const RAW_ADDRESS_LEN: usize = 33;
const USER_FRIENDLY_ADDRESS_LEN: usize = 36;

fn crc16(slice: &[u8]) -> u16 {
    Crc::<u16>::new(&crc::CRC_16_XMODEM).checksum(slice)
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Address {
    bytes: RawBytes,
}

impl Address {
    pub fn new(workchain: Workchain, hash_part: HashPart) -> Self {
        let mut bytes = [0u8; RAW_ADDRESS_LEN];
        bytes[0] = workchain as i8 as u8;
        bytes[1..].copy_from_slice(&hash_part);
        Self { bytes }
    }

    pub fn workchain(&self) -> Workchain {
        self.bytes[0] as i8 as i32
    }

    pub fn hash_part(&self) -> &HashPart {
        self.bytes[1..].try_into().unwrap()
    }

    pub fn try_parse_base64(base64: &str) -> Option<Self> {
        let bytes = decode_base64_url(base64).or_else(|_| decode_base64_no_pad(base64)).ok()?;
        if bytes.len() != USER_FRIENDLY_ADDRESS_LEN {
            return None;
        }
        let expected_crc = u16::from_be_bytes(bytes[34..36].try_into().ok()?);
        if expected_crc != crc16(&bytes[..34]) {
            return None;
        }
        let raw_bytes: RawBytes = bytes[1..RAW_ADDRESS_LEN + 1].try_into().ok()?;
        Some(Self { bytes: raw_bytes })
    }

    pub fn try_parse_hex(hex_str: &str) -> Option<Self> {
        let (workchain, hash_part) = hex_str.split_once(':')?;
        let workchain = workchain.parse::<i32>().ok()?;
        let hash_part: HashPart = hex::decode(hash_part).ok()?.try_into().ok()?;
        Some(Self::new(workchain, hash_part))
    }

    pub fn parse(value: &str) -> Result<Self, SignerError> {
        <Self as AddressTrait>::try_parse(value).ok_or_else(|| SignerError::invalid_input("invalid TON address"))
    }

    pub fn ensure_matches(claimed: Option<&str>, actual: &str) -> Result<(), SignerError> {
        let Some(claimed) = claimed.filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        if Self::parse(claimed)? != Self::parse(actual)? {
            return Err(SignerError::invalid_input("TON from does not match signer address"));
        }
        Ok(())
    }

    fn encode_user_friendly(&self) -> String {
        let mut buffer = [0u8; USER_FRIENDLY_ADDRESS_LEN];

        buffer[0] = USER_FRIENDLY_FLAG;
        buffer[1..RAW_ADDRESS_LEN + 1].copy_from_slice(&self.bytes);

        let crc = crc16(&buffer[..RAW_ADDRESS_LEN + 1]);
        buffer[34] = ((crc >> 8) & 0xFF) as u8;
        buffer[35] = (crc & 0xFF) as u8;

        encode_base64_url(&buffer)
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        <Self as AddressTrait>::from_str(address)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", <Self as AddressTrait>::encode(self))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Address")
            .field("workchain", &self.workchain())
            .field("hash_part", &hex::encode(self.hash_part()))
            .finish()
    }
}

impl AddressTrait for Address {
    fn try_parse(address: &str) -> Option<Self> {
        Self::try_parse_base64(address).or_else(|| Self::try_parse_hex(address))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn encode(&self) -> String {
        self.encode_user_friendly()
    }
}

pub fn hex_to_base64_address(hex_str: &str) -> Option<String> {
    Address::try_parse_hex(hex_str).map(|address| address.encode())
}

pub fn base64_to_hex_address(base64_str: &str) -> Option<String> {
    Address::try_parse_base64(base64_str).map(|address| format!("{}:{}", address.workchain(), hex::encode(address.hash_part())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let hex = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let encoded = "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8";
        let address = Address::try_parse_hex(hex).unwrap();

        assert_eq!(address.encode(), encoded);
        assert_eq!(<Address as AddressTrait>::from_str(hex).unwrap(), address);
        assert_eq!(<Address as AddressTrait>::from_str(encoded).unwrap(), address);
        assert_eq!(Address::try_parse(encoded), Some(address));
        assert!(Address::is_valid(encoded));
        assert_eq!(address.as_bytes().len(), RAW_ADDRESS_LEN);
        assert_eq!(address.workchain(), 0);
        assert_eq!(hex::encode(address.hash_part()), "8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
    }

    #[test]
    fn test_hex_to_base64_address() {
        let addr = "0:8c50a91220a5ccf086a1b2113b1a78787555f02b20d3fa6e97ba1acd710dbdaa";
        let result = hex_to_base64_address(addr).unwrap();

        assert_eq!(result, "EQCMUKkSIKXM8IahshE7Gnh4dVXwKyDT-m6XuhrNcQ29qvOh");
    }

    #[test]
    fn test_invalid_addresses() {
        assert!(Address::try_parse_hex("invalid").is_none());
        assert!(Address::try_parse_hex("abc:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78").is_none());
        assert!(Address::try_parse_hex("0:invalid_hex").is_none());
        assert!(Address::try_parse_hex("0:abcd1234").is_none());
        assert!(!Address::is_valid("invalid"));
        assert!(Address::try_parse("invalid").is_none());
    }

    #[test]
    fn test_base64_to_hex_address() {
        let base64 = "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8";
        let hex = base64_to_hex_address(base64).unwrap();

        assert_eq!(hex, "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
    }

    #[test]
    fn test_from_base64_url() {
        let addr = Address::try_parse_base64("UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg").unwrap();

        assert_eq!(addr.workchain(), 0);
        assert_eq!(hex::encode(addr.hash_part()), "58d5c54fbb8488af7eaad0cdc759ca8f6ff79fc9555106c1339b037ec0a40347");
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_hex = "0:0e97797708411c29a3cb1f3f810ef4f83f41d990838f7f93ce7082c4ff9aa026";
        let base64 = hex_to_base64_address(original_hex).unwrap();
        let decoded_hex = base64_to_hex_address(&base64).unwrap();

        assert_eq!(original_hex, decoded_hex);
    }
}
