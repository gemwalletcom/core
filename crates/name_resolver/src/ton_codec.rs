use crate::codec::Codec;

use gem_ton::address::{Address, base64_to_hex_address};
use std::error::Error;

pub struct TonCodec {}

impl Codec for TonCodec {
    fn decode(string: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        if let Ok(address) = Address::from_hex_str(string) {
            return Ok(address.get_hash_part().to_vec());
        }

        let hex = base64_to_hex_address(string.to_string())?;
        let address = Address::from_hex_str(&hex)?;
        Ok(address.get_hash_part().to_vec())
    }

    /// Encode to master chain base64 address
    fn encode(bytes: Vec<u8>) -> Result<String, Box<dyn Error + Send + Sync>> {
        let hash_part: [u8; 32] = {
            // raw hex address is 33 bytes
            if bytes.len() == 66 {
                let decoded = hex::decode(&bytes[2..])?;
                decoded.as_slice().try_into()?
            } else {
                bytes.as_slice().try_into()?
            }
        };
        let address = Address::new(0, hash_part);
        Ok(address.to_base64_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_encode() {
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let address = TonCodec::encode(raw.as_bytes().to_vec()).unwrap();

        assert_eq!(address, "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8");
    }

    #[test]
    fn test_decode() {
        let string = "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8";
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let bytes = TonCodec::decode(string).unwrap();
        let bytes2 = TonCodec::decode(raw).unwrap();

        assert_eq!(bytes, bytes2);
        assert_eq!(hex::encode(bytes), "8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
    }
}
