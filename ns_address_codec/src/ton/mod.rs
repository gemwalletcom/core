use ton_smart_contract_address::{RawAddress, UserFriendlyAddress, UserFriendlyFlag};

use crate::codec::Codec;

pub struct TonCodec {}

impl Codec for TonCodec {
    fn decode(string: &str) -> Vec<u8> {
        let r = UserFriendlyAddress::from_user_friendly_str(string);
        match r {
            Ok(address) => {
                let mut id = address.account_id().to_vec();
                // drop crc checksum
                let len = id.len() -2;
                id.truncate(len);
                id
            },
            Err(_) => Vec::new(),
        }
    }

    fn encode(bytes: Vec<u8>) -> String {
        let raw = std::str::from_utf8(&bytes).unwrap_or("");
        let r = RawAddress::from_raw_str(raw);
        match r {
            Ok(address) => {
                address.to_user_friendly_str(UserFriendlyFlag::Bounceable)
            },
            Err(_) => {
                "".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hex;
    use super::*;

    #[test]
    fn test_encode() {
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let address = TonCodec::encode(raw.as_bytes().to_vec());

        assert_eq!(address, "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8");
    }

    #[test]
    fn test_decode() {
        let string = "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8";
        let bytes = TonCodec::decode(string);

        assert_eq!(hex::encode(bytes), "8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
    }
}
