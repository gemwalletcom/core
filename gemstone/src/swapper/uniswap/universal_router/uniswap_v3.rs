use crate::swapper::{uniswap::v3::UniversalRouterProvider, SwapProvider, SwapProviderId};
use gem_evm::uniswap::{
    deployment::v3::{get_uniswap_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct UniswapUniversalRouter {
    pub provider: SwapProvider,
}

impl Default for UniswapUniversalRouter {
    fn default() -> Self {
        Self {
            provider: SwapProvider::new(SwapProviderId::UniswapV3),
        }
    }
}

impl UniversalRouterProvider for UniswapUniversalRouter {
    fn provider(&self) -> &SwapProvider {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_uniswap_router_deployment_by_chain(chain)
    }
}
