mod explorer;
mod model;

pub use explorer::MayanExplorer;
pub use model::{MayanClientStatus, MayanTransactionResult};

use primitives::Chain;
use std::str::FromStr;

/// Maps Mayan chain names to Chain enum
pub fn map_mayan_chain_to_chain(chain_name: &str) -> Option<Chain> {
    match chain_name {
        "bsc" => Some(Chain::SmartChain),
        "avalanche" => Some(Chain::AvalancheC),
        _ => Chain::from_str(chain_name).ok(),
    }
}
