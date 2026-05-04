use primitives::Address as AddressTrait;

use crate::pubkey::Pubkey;

pub fn validate_address(address: &str) -> bool {
    Pubkey::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_address() {
        let address = "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2";
        let parsed = Pubkey::try_parse(address).unwrap();

        assert!(validate_address(address));
        assert_eq!(parsed.as_bytes().len(), 32);
        assert_eq!(parsed.encode(), address);
        assert!(!validate_address("invalid"));
    }
}
