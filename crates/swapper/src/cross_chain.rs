use primitives::Transaction;

use crate::SwapperProvider;
use crate::across::AcrossCrossChain;
use crate::thorchain::ThorchainCrossChain;

pub trait CrossChainProvider: Send + Sync {
    fn provider(&self) -> SwapperProvider;
    fn is_swap(&self, transaction: &Transaction) -> bool;
}

const PROVIDERS: [&dyn CrossChainProvider; 2] = [&ThorchainCrossChain, &AcrossCrossChain];

pub fn swap_provider(transaction: &Transaction) -> Option<SwapperProvider> {
    PROVIDERS.iter().find(|p| p.is_swap(transaction)).map(|p| p.provider())
}

pub fn is_cross_chain_swap(transaction: &Transaction) -> bool {
    swap_provider(transaction).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_thorchain_swap_detected() {
        let transaction = Transaction {
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_thorchain_non_swap_memo() {
        assert!(!is_cross_chain_swap(&Transaction {
            memo: Some("ADD:ETH.ETH:0x123".to_string()),
            ..Transaction::mock()
        }));
    }

    #[test]
    fn test_no_memo() {
        assert!(swap_provider(&Transaction::mock()).is_none());
    }

    #[test]
    fn test_across_swap_detected() {
        let transaction = Transaction {
            to: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Across));
    }

    #[test]
    fn test_across_swap_case_insensitive() {
        let transaction = Transaction {
            to: "0x5c7bcd6e7de5423a257d81b442095a1a6ced35c5".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Across));
    }

    #[test]
    fn test_across_unsupported_chain() {
        let transaction = Transaction {
            asset_id: Chain::Fantom.as_asset_id(),
            to: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5".to_string(),
            ..Transaction::mock()
        };
        assert!(swap_provider(&transaction).is_none());
    }

    #[test]
    fn test_across_arbitrum() {
        let transaction = Transaction {
            asset_id: Chain::Arbitrum.as_asset_id(),
            to: "0xe35e9842fceaca96570b734083f4a58e8f7c5f2a".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Across));
    }

    #[test]
    fn test_thorchain_takes_priority_over_across() {
        let transaction = Transaction {
            to: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5".to_string(),
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_non_evm_chains_no_panic() {
        let btc = Transaction {
            asset_id: Chain::Bitcoin.as_asset_id(),
            to: "bc1qaddress".to_string(),
            ..Transaction::mock()
        };
        let sol = Transaction {
            asset_id: Chain::Solana.as_asset_id(),
            to: "So1111111111111111111111111111111111111111".to_string(),
            ..Transaction::mock()
        };
        assert!(swap_provider(&btc).is_none());
        assert!(swap_provider(&sol).is_none());
    }

    #[test]
    fn test_thorchain_swap_no_to_address() {
        let transaction = Transaction {
            asset_id: Chain::Litecoin.as_asset_id(),
            to: String::new(),
            memo: Some("=:s:0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4:0/1/0:g1:50".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_thorchain_evm_router_detected() {
        let transaction = Transaction {
            to: "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Thorchain));
    }

    #[test]
    fn test_thorchain_evm_data_detected() {
        let data = "0x3d3a623a626331713965797870616730777875386a74756b7a747a6b636876637a65793039616134397632326c353a302f312f303a67313a3530";
        let transaction = Transaction {
            to: "0xdfb89f7b854b79fdac99ddeb55921349ca649def".to_string(),
            data: Some(data.to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider(&transaction), Some(SwapperProvider::Thorchain));
    }
}
