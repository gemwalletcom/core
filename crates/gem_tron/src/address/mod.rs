pub mod serializer;

use alloy_primitives::Address;

const ADDRESS_PREFIX: u8 = 0x41;
const ADDRESS_LEN: usize = 20;
const PREFIXED_ADDRESS_LEN: usize = ADDRESS_LEN + 1;

pub struct TronAddress;

impl TronAddress {
    pub fn from_hex(hex_value: &str) -> Option<String> {
        let decoded = hex::decode(hex_value).ok()?;
        Some(bs58::encode(decoded).with_check().into_string())
    }

    pub fn to_hex(address: &str) -> Option<String> {
        let decoded = bs58::decode(address).with_check(None).into_vec().ok()?;
        Some(hex::encode(decoded))
    }

    pub fn to_addr(address: &str) -> Option<Address> {
        let decoded = bs58::decode(address).with_check(None).into_vec().ok()?;
        match decoded.len() {
            PREFIXED_ADDRESS_LEN if decoded[0] == ADDRESS_PREFIX => {
                let mut addr = [0u8; ADDRESS_LEN];
                addr.copy_from_slice(&decoded[1..PREFIXED_ADDRESS_LEN]);
                Some(Address::from(addr))
            }
            ADDRESS_LEN => {
                let mut addr = [0u8; ADDRESS_LEN];
                addr.copy_from_slice(&decoded[..ADDRESS_LEN]);
                Some(Address::from(addr))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(
            TronAddress::from_hex("4159f3440fd40722f716144e4490a4de162d3b3fcb").unwrap(),
            "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH".to_string()
        );
        assert_eq!(
            TronAddress::from_hex("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap(),
            "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1".to_string()
        );
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(
            TronAddress::to_hex("TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1"),
            Some("41357a7401a0f0c2d4a44a1881a0c622f15d986291".to_string())
        );
    }

    #[test]
    fn test_to_addr_from_base58() {
        let expected = Address::from_slice(&hex::decode("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap());
        assert_eq!(TronAddress::to_addr("TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1").unwrap(), expected);
    }
}
