mod commands;
mod path;
mod quoter_v2;

pub mod provider;
pub use provider::UniswapV3;

use crate::swapper::SwapperProviderType;
use gem_evm::uniswap::{FeeTier, deployment::v3::V3Deployment};
use primitives::Chain;
use std::fmt::Debug;

pub trait UniversalRouterProvider: Send + Sync + Debug {
    fn provider(&self) -> &SwapperProviderType;
    fn get_tiers(&self) -> Vec<FeeTier>;
    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment>;
}

const DEFAULT_SWAP_GAS_LIMIT: u64 = 500_000; // gwei
