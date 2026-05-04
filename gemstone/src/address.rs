use crate::GemstoneError;
use gem_bitcoin::models::address::Address as BitcoinAddress;
use primitives::{Chain, ChainAddress, ChainType};
use std::sync::Arc;

#[derive(uniffi::Object)]
pub struct GemChainAddress {
    inner: ChainAddress,
}

#[uniffi::export]
impl GemChainAddress {
    #[uniffi::constructor]
    pub fn new(address: String, chain: Chain) -> Result<Arc<Self>, GemstoneError> {
        if !validate_address(&address, chain) {
            return Err(GemstoneError::AnyError {
                msg: format!("Invalid address for {chain}"),
            });
        }
        Ok(Arc::new(Self {
            inner: ChainAddress::new(chain, address),
        }))
    }

    pub fn address(&self) -> String {
        self.inner.address().to_string()
    }
}

#[uniffi::export]
pub fn validate_address(address: &str, chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::HyperCore => gem_evm::validate_address(address),
        ChainType::Solana => gem_solana::validate_address(address),
        ChainType::Cosmos => gem_cosmos::validate_address(address, chain),
        ChainType::Ton => gem_ton::validate_address(address),
        ChainType::Tron => gem_tron::validate_address(address),
        ChainType::Aptos => gem_aptos::validate_address(address),
        ChainType::Sui => gem_sui::validate_address(address),
        ChainType::Near => gem_near::validate_address(address),
        ChainType::Stellar => gem_stellar::validate_address(address),
        ChainType::Algorand => gem_algorand::validate_address(address),
        ChainType::Bitcoin | ChainType::Xrp | ChainType::Polkadot | ChainType::Cardano => false,
    }
}

#[uniffi::export]
pub fn short_address(address: &str, chain: Chain) -> String {
    match chain {
        Chain::BitcoinCash => BitcoinAddress::new(address, Chain::BitcoinCash).short().to_string(),
        _ => address.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_address_validation() {
        assert!(validate_address("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a", Chain::Ethereum));
        assert!(!validate_address("0X5615e8ab93b9d695b6d4d6545f7792aa59e1069a", Chain::Ethereum));
        assert!(validate_address("cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj", Chain::Cosmos));
        assert!(validate_address("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2", Chain::Solana));
    }

    #[test]
    fn test_new_returns_err_for_invalid() {
        assert!(GemChainAddress::new("invalid".to_string(), Chain::Ethereum).is_err());
        assert!(GemChainAddress::new("0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a".to_string(), Chain::Ethereum).is_ok());
    }

    #[test]
    fn test_short_address_bitcoincash() {
        let prefixed = "bitcoincash:qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70";
        let stripped = "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70";

        for input in [prefixed, stripped] {
            assert_eq!(short_address(input, Chain::BitcoinCash), stripped);
        }
    }

    #[test]
    fn test_short_address_passthrough() {
        let eth = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a";
        assert_eq!(short_address(eth, Chain::Ethereum), eth);
    }
}
