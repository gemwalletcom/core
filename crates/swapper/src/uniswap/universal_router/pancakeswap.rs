use crate::{ProviderType, SwapperProvider, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_pancakeswap_router_deployment_by_chain},
};
use primitives::Chain;

#[derive(Debug)]
pub struct PancakeSwapUniversalRouter {
    pub provider: ProviderType,
}

impl Default for PancakeSwapUniversalRouter {
    fn default() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::PancakeswapV3),
        }
    }
}

impl UniversalRouterProvider for PancakeSwapUniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::TwoThousandFiveHundred, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_pancakeswap_router_deployment_by_chain(chain)
    }
}
