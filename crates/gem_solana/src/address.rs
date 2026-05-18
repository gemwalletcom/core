use solana_primitives::Pubkey;

pub fn validate_address(address: &str) -> bool {
    Pubkey::from_base58(address).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_address() {
        let address = "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2";
        let pubkey = Pubkey::from_base58(address).unwrap();

        assert!(validate_address(address));
        assert_eq!(pubkey.as_bytes().len(), 32);
        assert_eq!(pubkey.to_base58(), address);
        assert!(!validate_address("invalid"));
    }
}
