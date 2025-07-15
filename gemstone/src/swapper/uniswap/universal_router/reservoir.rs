use crate::swapper::{uniswap::v3::UniversalRouterProvider, SwapperProvider, SwapperProviderType};
use gem_evm::uniswap::{
    deployment::v3::{get_reservoir_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct ReservoirUniversalRouter {
    pub provider: SwapperProviderType,
}

impl Default for ReservoirUniversalRouter {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Reservoir),
        }
    }
}

impl UniversalRouterProvider for ReservoirUniversalRouter {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_reservoir_deployment_by_chain(chain)
    }
}
