use primitives::Address as AddressTrait;
use solana_primitives::{Pubkey, SolanaError};

pub struct SolanaAddress(Pubkey);

impl From<SolanaAddress> for Pubkey {
    fn from(value: SolanaAddress) -> Self {
        value.0
    }
}

impl SolanaAddress {
    pub fn parse(address: &str) -> Result<Self, SolanaError> {
        Pubkey::from_base58(address).map(Self)
    }
}

impl AddressTrait for SolanaAddress {
    fn try_parse(address: &str) -> Option<Self> {
        Pubkey::from_base58(address).ok().map(Self)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    fn encode(&self) -> String {
        self.0.to_base58()
    }
}

pub fn validate_address(address: &str) -> bool {
    SolanaAddress::is_valid(address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_address() {
        let string = "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2";
        let address = SolanaAddress::try_parse(string).unwrap();

        assert!(validate_address(string));
        assert_eq!(address.as_bytes().len(), 32);
        assert_eq!(address.encode(), string);
        assert_eq!(SolanaAddress::parse(string).unwrap().encode(), string);
        assert!(SolanaAddress::parse("invalid").is_err());
        assert!(!validate_address("invalid"));
    }
}
