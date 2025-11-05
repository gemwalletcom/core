pub mod agent;
pub mod config;
pub mod core;
pub mod models;
pub mod provider;
pub mod rpc;

#[cfg(feature = "signer")]
pub mod signer;

use primitives::Chain;

pub fn is_bridge_swap(from_chain: Chain, to_chain: Chain) -> bool {
    (from_chain == Chain::HyperCore && to_chain == Chain::Hyperliquid) || (from_chain == Chain::Hyperliquid && to_chain == Chain::HyperCore)
}

pub fn is_spot_swap(from_chain: Chain, to_chain: Chain) -> bool {
    from_chain == Chain::HyperCore && to_chain == Chain::HyperCore
}
