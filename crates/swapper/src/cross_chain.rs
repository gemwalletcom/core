use crate::SwapperProvider;
use crate::across::AcrossCrossChain;
use crate::thorchain::ThorchainCrossChain;
use primitives::Chain;

pub trait CrossChainProvider: Send + Sync {
    fn provider(&self) -> SwapperProvider;
    fn is_swap(&self, chain: &Chain, to_address: Option<&str>, memo: Option<&str>) -> bool;
}

const PROVIDERS: [&dyn CrossChainProvider; 2] = [&ThorchainCrossChain, &AcrossCrossChain];

pub fn swap_provider(chain: &Chain, to_address: Option<&str>, memo: Option<&str>) -> Option<SwapperProvider> {
    PROVIDERS.iter().find(|p| p.is_swap(chain, to_address, memo)).map(|p| p.provider())
}

pub fn is_cross_chain_swap(chain: &Chain, to_address: &str, memo: Option<&str>) -> bool {
    swap_provider(chain, Some(to_address), memo).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thorchain_swap_detected() {
        let memo = "=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0";
        assert_eq!(
            swap_provider(&Chain::Ethereum, Some("0x0000000000000000000000000000000000000000"), Some(memo)),
            Some(SwapperProvider::Thorchain),
        );
    }

    #[test]
    fn test_thorchain_non_swap_memo() {
        assert!(!is_cross_chain_swap(
            &Chain::Ethereum,
            "0x0000000000000000000000000000000000000000",
            Some("ADD:ETH.ETH:0x123"),
        ));
    }

    #[test]
    fn test_no_memo() {
        assert!(swap_provider(&Chain::Ethereum, Some("0x0000000000000000000000000000000000000000"), None).is_none());
    }

    #[test]
    fn test_across_swap_detected() {
        assert_eq!(
            swap_provider(&Chain::Ethereum, Some("0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5"), None),
            Some(SwapperProvider::Across),
        );
    }

    #[test]
    fn test_across_swap_case_insensitive() {
        assert!(is_cross_chain_swap(&Chain::Ethereum, "0x5c7bcd6e7de5423a257d81b442095a1a6ced35c5", None));
    }

    #[test]
    fn test_across_unsupported_chain() {
        assert!(!is_cross_chain_swap(&Chain::Fantom, "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5", None));
    }

    #[test]
    fn test_across_arbitrum() {
        assert_eq!(
            swap_provider(&Chain::Arbitrum, Some("0xe35e9842fceaca96570b734083f4a58e8f7c5f2a"), None),
            Some(SwapperProvider::Across),
        );
    }

    #[test]
    fn test_thorchain_takes_priority_over_across() {
        let memo = "=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0";
        assert_eq!(
            swap_provider(&Chain::Ethereum, Some("0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5"), Some(memo)),
            Some(SwapperProvider::Thorchain),
        );
    }

    #[test]
    fn test_non_evm_chains_no_panic() {
        assert!(!is_cross_chain_swap(&Chain::Bitcoin, "bc1qaddress", None));
        assert!(!is_cross_chain_swap(&Chain::Solana, "So1111111111111111111111111111111111111111", None));
        assert!(!is_cross_chain_swap(&Chain::Ton, "EQAddress", None));
        assert!(!is_cross_chain_swap(&Chain::Cosmos, "cosmos1address", None));
        assert!(!is_cross_chain_swap(&Chain::Sui, "0xaddress", None));
    }

    #[test]
    fn test_thorchain_swap_no_to_address() {
        let memo = "=:s:0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4:0/1/0:g1:50";
        assert_eq!(swap_provider(&Chain::Litecoin, None, Some(memo)), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_thorchain_evm_router_detected() {
        assert_eq!(
            swap_provider(&Chain::Ethereum, Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146"), None),
            Some(SwapperProvider::Thorchain),
        );
    }
}
