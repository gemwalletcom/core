use gem_evm::uniswap::{
    deployment::{get_pancakeswap_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

use super::{SwapProvider, UniversalRouterProvider};

#[derive(Debug)]
pub struct PancakeSwapUniversalRouter {}

impl UniversalRouterProvider for PancakeSwapUniversalRouter {
    fn provider(&self) -> SwapProvider {
        SwapProvider::PancakeSwapV3
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::TwoThousandFiveHundred, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_pancakeswap_router_deployment_by_chain(chain)
    }
}
