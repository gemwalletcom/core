mod commands;
mod path;
mod quoter_v2;

pub mod provider;
pub use provider::UniswapV3;

use crate::swapper::SwapProvider;
use gem_evm::uniswap::{deployment::v3::V3Deployment, FeeTier};
use primitives::Chain;
use std::fmt::Debug;

pub trait UniversalRouterProvider: Send + Sync + Debug {
    fn provider(&self) -> SwapProvider;
    fn get_tiers(&self) -> Vec<FeeTier>;
    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment>;
}

static DEFAULT_DEADLINE: u64 = 3600;
const DEFAULT_SWAP_GAS_LIMIT: u64 = 500_000; // gwei
