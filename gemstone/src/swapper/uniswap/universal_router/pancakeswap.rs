use crate::swapper::{uniswap::v3::UniversalRouterProvider, SwapperProviderType, SwapperProvider};
use gem_evm::uniswap::{
    deployment::v3::{get_pancakeswap_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct PancakeSwapUniversalRouter {
    pub provider: SwapperProviderType,
}

impl Default for PancakeSwapUniversalRouter {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::PancakeswapV3),
        }
    }
}

impl UniversalRouterProvider for PancakeSwapUniversalRouter {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::TwoThousandFiveHundred, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_pancakeswap_router_deployment_by_chain(chain)
    }
}
