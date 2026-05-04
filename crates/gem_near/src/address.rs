use primitives::Address as AddressTrait;

pub struct NearAddress([u8; 32]);

impl AddressTrait for NearAddress {
    fn try_parse(address: &str) -> Option<Self> {
        hex::decode(address).ok()?.try_into().ok().map(Self)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        hex::encode(self.0)
    }
}

pub fn validate_address(address: &str) -> bool {
    NearAddress::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_address() {
        let address = "e3ac115fd911eb985ffd884ee60302c84dc94df52127ccde8d6fb97ad6d22945";
        let parsed = NearAddress::try_parse(address).unwrap();

        assert!(validate_address(address));
        assert_eq!(parsed.as_bytes().len(), 32);
        assert_eq!(parsed.encode(), address);
        assert!(!validate_address("invalid"));
        assert!(!validate_address("e3ac115fd911eb985ffd884ee60302c84dc94df52127ccde8d6fb97ad6d229"));
    }
}
