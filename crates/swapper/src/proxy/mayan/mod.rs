mod explorer;
mod model;

pub use explorer::MayanExplorer;
pub use model::{MayanClientStatus, MayanTransactionResult};

use primitives::Chain;
use std::str::FromStr;

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

/// Maps Mayan chain names to Chain enum
pub fn map_mayan_chain_to_chain(chain_name: &str) -> Option<Chain> {
    match chain_name {
        "bsc" => Some(Chain::SmartChain),
        "avalanche" => Some(Chain::AvalancheC),
        _ => Chain::from_str(chain_name).ok(),
    }
}
