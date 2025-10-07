use crate::{SwapperProvider, ProviderType, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_oku_deployment_by_chain},
};
use primitives::Chain;

#[derive(Debug)]
pub struct OkuUniversalRouter {
    pub provider: ProviderType,
}

impl Default for OkuUniversalRouter {
    fn default() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Oku),
        }
    }
}

impl UniversalRouterProvider for OkuUniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_oku_deployment_by_chain(chain)
    }
}
