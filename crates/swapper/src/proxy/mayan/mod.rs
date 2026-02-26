mod explorer;
mod model;
mod price;

pub use explorer::MayanExplorer;
pub use model::{MayanChain, MayanClientStatus, MayanTransactionResult};
pub use price::MayanPrice;

use crate::SwapperProvider;
use crate::cross_chain::CrossChainProvider;
use gem_evm::ethereum_address_checksum;
use primitives::{AssetId, Chain, ChainType, Transaction};

const MAYAN_FORWARDER: &str = "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2";
const MAYAN_MCTP: &str = "0x875d6d37EC55c8cF220B9E5080717549d8Aa8EcA";
const MAYAN_SWIFT: &str = "0xC38e4e6A15593f908255214653d3D947CA1c2338";
const MAYAN_FULFILL_HELPER: &str = "0xBC0663ef502F0Ee9676626ED5B418037252eFeb2";

pub const MAYAN_CONTRACTS: [&str; 4] = [MAYAN_FORWARDER, MAYAN_MCTP, MAYAN_SWIFT, MAYAN_FULFILL_HELPER];

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub struct MayanCrossChain;

impl CrossChainProvider for MayanCrossChain {
    fn provider(&self) -> SwapperProvider {
        SwapperProvider::Mayan
    }

    fn is_swap(&self, transaction: &Transaction) -> bool {
        MAYAN_CONTRACTS.contains(&transaction.to.as_str())
    }
}

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
    let is_native = token_address == ZERO_ADDRESS || chain.config().denom.is_some_and(|d| d == token_address);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_asset_id_native_eth() {
        let result = resolve_asset_id(Chain::Ethereum, ZERO_ADDRESS);
        assert_eq!(result, Some(AssetId::from_chain(Chain::Ethereum)));
    }

    #[test]
    fn test_resolve_asset_id_native_sui() {
        let result = resolve_asset_id(Chain::Sui, "0x2::sui::SUI");
        assert_eq!(result, Some(AssetId::from_chain(Chain::Sui)));
    }

    #[test]
    fn test_resolve_asset_id_native_solana_zero_address() {
        let result = resolve_asset_id(Chain::Solana, ZERO_ADDRESS);
        assert_eq!(result, Some(AssetId::from_chain(Chain::Solana)));
    }

    #[test]
    fn test_resolve_asset_id_evm_token_checksummed() {
        let result = resolve_asset_id(Chain::Ethereum, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        assert_eq!(result, Some(AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")));
    }

    #[test]
    fn test_resolve_asset_id_base_usdc() {
        let result = resolve_asset_id(Chain::Base, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913");
        assert_eq!(result, Some(AssetId::from_token(Chain::Base, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")));
    }

    #[test]
    fn test_resolve_asset_id_solana_token() {
        let result = resolve_asset_id(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
        assert_eq!(result, Some(AssetId::from_token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")));
    }

    #[test]
    fn test_wormhole_chain_id_to_chain() {
        assert_eq!(wormhole_chain_id_to_chain(1), Some(Chain::Solana));
        assert_eq!(wormhole_chain_id_to_chain(2), Some(Chain::Ethereum));
        assert_eq!(wormhole_chain_id_to_chain(21), Some(Chain::Sui));
        assert_eq!(wormhole_chain_id_to_chain(23), Some(Chain::Arbitrum));
        assert_eq!(wormhole_chain_id_to_chain(30), Some(Chain::Base));
        assert_eq!(wormhole_chain_id_to_chain(9999), None);
    }

    #[test]
    fn test_resolve_metadata_eth_to_sui_completed() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/eth_to_sui_swift.json")).unwrap();
        assert_eq!(result.client_status, MayanClientStatus::Completed);

        let from_chain = result.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain).unwrap();
        let to_chain = result.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain).unwrap();

        assert_eq!(from_chain, Chain::Ethereum);
        assert_eq!(to_chain, Chain::Sui);

        let from_asset = resolve_asset_id(from_chain, &result.from_token_address).unwrap();
        let to_asset = resolve_asset_id(to_chain, &result.to_token_address).unwrap();

        assert_eq!(from_asset, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(to_asset, AssetId::from_chain(Chain::Sui));
        assert_eq!(result.from_amount64.unwrap(), "18124254");
    }

    #[test]
    fn test_resolve_metadata_base_usdc_to_arb_usdc() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/mctp_pending.json")).unwrap();

        let from_chain = result.from_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain).unwrap();
        let to_chain = result.to_token_chain.parse::<u16>().ok().and_then(wormhole_chain_id_to_chain).unwrap();

        assert_eq!(from_chain, Chain::Base);
        assert_eq!(to_chain, Chain::Arbitrum);

        let from_asset = resolve_asset_id(from_chain, &result.from_token_address).unwrap();
        let to_asset = resolve_asset_id(to_chain, &result.to_token_address).unwrap();

        assert_eq!(from_asset, AssetId::from_token(Chain::Base, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"));
        assert_eq!(to_asset, AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"));
        assert_eq!(result.from_amount64.unwrap(), "529066169");
    }

    #[test]
    fn test_resolve_metadata_refunded_no_metadata() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/swift_refunded.json")).unwrap();
        assert_eq!(result.client_status, MayanClientStatus::Refunded);
        assert_eq!(result.client_status.swap_status(), primitives::swap::SwapStatus::Failed);
    }
}
