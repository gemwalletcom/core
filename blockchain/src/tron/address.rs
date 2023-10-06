

pub struct TronAddress {
    
}

impl TronAddress {
    pub fn from_hex(hex_value: &str) -> Option<String>{
        let decoded = hex::decode(hex_value).ok()?;
        let encoded = bs58::encode(decoded).with_check().into_string();
        Some(encoded)
    }

    #[allow(dead_code)]
    pub fn from_base58(address: &str) -> Option<String> {
        let decoded = bs58::decode(address).with_check(None).into_vec().ok()?;
        let hex = hex::encode(decoded);
        Some(hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(TronAddress::from_hex("4159f3440fd40722f716144e4490a4de162d3b3fcb").unwrap(), "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH".to_string());
        assert_eq!(TronAddress::from_hex("41357a7401a0f0c2d4a44a1881a0c622f15d986291").unwrap(), "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1".to_string());
    }

    #[test]
    fn test_from_base58() {
        assert_eq!(TronAddress::from_base58("TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1"), Some("41357a7401a0f0c2d4a44a1881a0c622f15d986291".to_string()));
    }
}