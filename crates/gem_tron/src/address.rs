pub struct TronAddress {}

impl TronAddress {
    pub fn from_hex(hex_value: &str) -> Option<String> {
        let decoded = hex::decode(hex_value).ok()?;
        Some(bs58::encode(decoded).with_check().into_string())
    }

    #[allow(dead_code)]
    pub fn to_hex(address: &str) -> Option<String> {
        let decoded = bs58::decode(address).with_check(None).into_vec().ok()?;
        Some(hex::encode(decoded))
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
}
