use crate::{SwapperProvider, ProviderType, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_aerodrome_router_deployment_by_chain},
};
use primitives::Chain;

#[derive(Debug)]
pub struct AerodromeUniversalRouter {
    pub provider: ProviderType,
}

impl Default for AerodromeUniversalRouter {
    fn default() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Aerodrome),
        }
    }
}

impl UniversalRouterProvider for AerodromeUniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![
            FeeTier::Hundred,
            FeeTier::FourHundred,
            FeeTier::FiveHundred,
            FeeTier::ThreeThousand,
            FeeTier::TenThousand,
        ]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_aerodrome_router_deployment_by_chain(chain)
    }
}
