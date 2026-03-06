mod explorer;
mod model;
mod price;

pub use explorer::MayanExplorer;
pub use model::{MayanChain, MayanClientStatus, MayanTransactionResult};
pub use price::MayanPrice;

use crate::{SwapResult, SwapperProvider, asset::EVM_ZERO_ADDRESS};
use gem_evm::ethereum_address_checksum;
use gem_solana::WSOL_TOKEN_ADDRESS;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, AssetId, Chain, ChainType, TransactionSwapMetadata};

const MAYAN_FORWARDER: &str = "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2";
const MAYAN_MCTP: &str = "0x875d6d37EC55c8cF220B9E5080717549d8Aa8EcA";
const MAYAN_SWIFT: &str = "0xC38e4e6A15593f908255214653d3D947CA1c2338";
const MAYAN_FULFILL_HELPER: &str = "0xBC0663ef502F0Ee9676626ED5B418037252eFeb2";

pub const MAYAN_DEPOSIT_CONTRACTS: [&str; 3] = [MAYAN_FORWARDER, MAYAN_MCTP, MAYAN_SWIFT];
pub const MAYAN_SEND_CONTRACTS: [&str; 1] = [MAYAN_FULFILL_HELPER];

/// https://wormhole.com/docs/products/reference/chain-ids
pub fn wormhole_chain_id_to_chain(chain_id: u16) -> Option<Chain> {
    match chain_id {
        1 => Some(Chain::Solana),
        2 => Some(Chain::Ethereum),
        4 => Some(Chain::SmartChain),
        5 => Some(Chain::Polygon),
        6 => Some(Chain::AvalancheC),
        10 => Some(Chain::Fantom),
        14 => Some(Chain::Celo),
        15 => Some(Chain::Near),
        21 => Some(Chain::Sui),
        22 => Some(Chain::Aptos),
        23 => Some(Chain::Arbitrum),
        24 => Some(Chain::Optimism),
        30 => Some(Chain::Base),
        38 => Some(Chain::Linea),
        39 => Some(Chain::Berachain),
        44 => Some(Chain::Unichain),
        45 => Some(Chain::World),
        47 => Some(Chain::Hyperliquid),
        48 => Some(Chain::Monad),
        58 => Some(Chain::Plasma),
        65000 => Some(Chain::HyperCore),
        _ => None,
    }
}

pub fn resolve_asset_id(chain: Chain, token_address: &str) -> Option<AssetId> {
    let is_native = token_address == EVM_ZERO_ADDRESS || token_address == WSOL_TOKEN_ADDRESS || chain.config().denom.is_some_and(|d| d == token_address);
    if is_native {
        return Some(AssetId::from_chain(chain));
    }
    let address = if chain.chain_type() == ChainType::Ethereum {
        ethereum_address_checksum(token_address).ok()?
    } else {
        token_address.to_string()
    };
    Some(AssetId::from_token(chain, &address))
}

fn resolve_value(chain: Chain, token_address: &str, amount: &str) -> Option<String> {
    let decimals = if resolve_asset_id(chain, token_address).is_some_and(|id| id.is_native()) {
        Asset::from_chain(chain).decimals
    } else {
        amount.find('.').map(|i| (amount.len() - i - 1) as i32).unwrap_or(0)
    };
    BigNumberFormatter::value_from_amount(amount, decimals as u32).ok()
}

pub fn map_swap_result(result: &MayanTransactionResult) -> SwapResult {
    let status = result.client_status.swap_status();

    let from_chain = result.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain);
    let to_chain = result.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain);

    let metadata = if result.client_status != MayanClientStatus::InProgress {
        from_chain.zip(to_chain).and_then(|(from_chain, to_chain)| {
            Some(TransactionSwapMetadata {
                from_asset: resolve_asset_id(from_chain, &result.from_token_address)?,
                from_value: resolve_value(from_chain, &result.from_token_address, &result.from_amount)?,
                to_asset: resolve_asset_id(to_chain, &result.to_token_address)?,
                to_value: resolve_value(to_chain, &result.to_token_address, &result.to_amount)?,
                provider: Some(SwapperProvider::Mayan.as_ref().to_string()),
            })
        })
    } else {
        None
    };

    SwapResult { status, metadata }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::swap::SwapStatus;

    fn result(json: &str) -> MayanTransactionResult {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_resolve_asset_id_native_eth() {
        assert_eq!(resolve_asset_id(Chain::Ethereum, EVM_ZERO_ADDRESS), Some(AssetId::from_chain(Chain::Ethereum)));
    }

    #[test]
    fn test_resolve_asset_id_native_sui() {
        assert_eq!(resolve_asset_id(Chain::Sui, "0x2::sui::SUI"), Some(AssetId::from_chain(Chain::Sui)));
    }

    #[test]
    fn test_resolve_asset_id_native_solana() {
        assert_eq!(resolve_asset_id(Chain::Solana, EVM_ZERO_ADDRESS), Some(AssetId::from_chain(Chain::Solana)));
        assert_eq!(resolve_asset_id(Chain::Solana, WSOL_TOKEN_ADDRESS), Some(AssetId::from_chain(Chain::Solana)));
    }

    #[test]
    fn test_resolve_asset_id_evm_token_checksummed() {
        assert_eq!(resolve_asset_id(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), Some(AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")));
    }

    #[test]
    fn test_wormhole_chain_id_to_chain() {
        assert_eq!(wormhole_chain_id_to_chain(1), Some(Chain::Solana));
        assert_eq!(wormhole_chain_id_to_chain(2), Some(Chain::Ethereum));
        assert_eq!(wormhole_chain_id_to_chain(21), Some(Chain::Sui));
        assert_eq!(wormhole_chain_id_to_chain(30), Some(Chain::Base));
        assert_eq!(wormhole_chain_id_to_chain(9999), None);
    }

    #[test]
    fn test_resolve_value() {
        // native: uses chain decimals (18 for ETH)
        assert_eq!(resolve_value(Chain::Ethereum, EVM_ZERO_ADDRESS, "0.01"), Some("10000000000000000".to_string()));
        // native: uses chain decimals (18 for POL), no decimal point
        assert_eq!(resolve_value(Chain::Polygon, EVM_ZERO_ADDRESS, "212"), Some("212000000000000000000".to_string()));
        // native: uses chain decimals (9 for SUI)
        assert_eq!(resolve_value(Chain::Sui, "0x2::sui::SUI", "7.534906306"), Some("7534906306".to_string()));
        // token: infers 6 decimals from amount string
        assert_eq!(resolve_value(Chain::Polygon, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f", "35.243141"), Some("35243141".to_string()));
        // token: infers 18 decimals from amount string
        assert_eq!(resolve_value(Chain::Base, "0xef5997c2cf2f6c138196f8a6203afc335206b3c1", "398.724622644505839482"), Some("398724622644505839482".to_string()));
        // token: infers 9 decimals from amount string
        assert_eq!(resolve_value(Chain::Solana, "CzFvsLdUazabdiu9TYXujj4EY495fG7VgJJ3vQs6bonk", "278080.608518046"), Some("278080608518046".to_string()));
    }

    #[test]
    fn test_map_swap_result_eth_to_sui() {
        assert_eq!(
            map_swap_result(&result(include_str!("../test/eth_to_sui_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Ethereum),
                    from_value: "10000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Sui),
                    to_value: "7534906306".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_pol_to_bnb() {
        assert_eq!(
            map_swap_result(&result(include_str!("../test/pol_to_bnb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Polygon),
                    from_value: "212000000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::SmartChain),
                    to_value: "33060513057817862".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_usdt_to_owb() {
        assert_eq!(
            map_swap_result(&result(include_str!("../test/usdt_to_owb_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::Polygon, "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"),
                    from_value: "35243141".to_string(),
                    to_asset: AssetId::from_token(Chain::Base, "0xEF5997c2cf2f6c138196f8A6203afc335206b3c1"),
                    to_value: "398724622644505839482".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_btcbr_to_radr() {
        assert_eq!(
            map_swap_result(&result(include_str!("../test/btcbr_to_radr_swift.json"))),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_token(Chain::SmartChain, "0x0cF8e180350253271f4b917CcFb0aCCc4862F262"),
                    from_value: "1500000000000000000000000000102072".to_string(),
                    to_asset: AssetId::from_token(Chain::Solana, "CzFvsLdUazabdiu9TYXujj4EY495fG7VgJJ3vQs6bonk"),
                    to_value: "278080608518046".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
    }

    #[test]
    fn test_map_swap_result_mctp_pending() {
        let result = map_swap_result(&result(include_str!("../test/mctp_pending.json")));
        assert_eq!(result.status, SwapStatus::Pending);
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_refunded() {
        assert_eq!(
            map_swap_result(&result(include_str!("../test/swift_refunded.json"))),
            SwapResult {
                status: SwapStatus::Failed,
                metadata: Some(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Ethereum),
                    from_value: "4000000000000000".to_string(),
                    to_asset: AssetId::from_chain(Chain::Solana),
                    to_value: "3342230".to_string(),
                    provider: Some("mayan".to_string()),
                }),
            }
        );
    }
}
