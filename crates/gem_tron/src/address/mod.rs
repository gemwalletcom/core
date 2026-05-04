pub mod serializer;

use alloy_primitives::Address;
use primitives::Address as AddressTrait;

const ADDRESS_PREFIX: u8 = 0x41;
const ADDRESS_LEN: usize = 20;
const PREFIXED_ADDRESS_LEN: usize = ADDRESS_LEN + 1;

pub struct TronAddress([u8; PREFIXED_ADDRESS_LEN]);

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
        let payload = match decoded.as_slice() {
            [ADDRESS_PREFIX, payload @ ..] => payload,
            payload => payload,
        };

        (payload.len() == ADDRESS_LEN).then(|| Address::from_slice(payload))
    }
}

impl AddressTrait for TronAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Self::to_addr(address).map(Self::from)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        bs58::encode(self.0).with_check().into_string()
    }
}

pub fn validate_address(address: &str) -> bool {
    TronAddress::is_valid(address)
}

impl From<Address> for TronAddress {
    fn from(address: Address) -> Self {
        let mut bytes = [0u8; PREFIXED_ADDRESS_LEN];
        bytes[0] = ADDRESS_PREFIX;
        bytes[1..].copy_from_slice(address.as_ref());
        Self(bytes)
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

    #[test]
    fn test_try_parse_normalizes_prefixed_and_unprefixed_payloads() {
        let prefixed = "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1";
        let payload = hex::decode("357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();
        let unprefixed = bs58::encode(&payload).with_check().into_string();
        let expected = hex::decode("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();

        assert!(validate_address(prefixed));
        assert!(validate_address(&unprefixed));
        assert_eq!(TronAddress::try_parse(prefixed).unwrap().as_bytes(), expected);
        assert_eq!(TronAddress::try_parse(&unprefixed).unwrap().as_bytes(), expected);
    }

    #[test]
    fn test_try_parse_rejects_wrong_prefix() {
        let mut decoded = hex::decode("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap();
        decoded[0] = 0x42;
        let address = bs58::encode(decoded).with_check().into_string();

        assert!(TronAddress::try_parse(&address).is_none());
        assert!(!validate_address(&address));
    }
}
