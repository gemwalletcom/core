use primitives::Address as AddressTrait;
use std::str::FromStr;
use sui_types::Address;

pub struct SuiAddress(Address);

impl AddressTrait for SuiAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Address::from_str(address).ok().map(Self)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn encode(&self) -> String {
        self.0.to_string()
    }
}

pub fn validate_address(address: &str) -> bool {
    SuiAddress::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sui_address() {
        let address = "0xada112cfb90b44ba889cc5d39ac2bf46281e4a91f7919c693bcd9b8323e81ed2";
        let parsed = SuiAddress::try_parse(address).unwrap();

        assert!(validate_address(address));
        assert_eq!(parsed.as_bytes().len(), 32);
        assert_eq!(parsed.encode(), address);
        assert!(!validate_address("invalid"));
    }
}
