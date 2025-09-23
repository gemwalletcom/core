use crate::swapper::{SwapperProvider, SwapperProviderType, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_reservoir_deployment_by_chain},
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
