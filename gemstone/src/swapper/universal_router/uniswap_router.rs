use gem_evm::uniswap::{
    deployment::v3::{get_uniswap_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

use super::{SwapProvider, UniversalRouterProvider};

#[derive(Debug)]
pub struct UniswapUniversalRouter {}

impl UniversalRouterProvider for UniswapUniversalRouter {
    fn provider(&self) -> SwapProvider {
        SwapProvider::UniswapV3
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_uniswap_router_deployment_by_chain(chain)
    }
}
