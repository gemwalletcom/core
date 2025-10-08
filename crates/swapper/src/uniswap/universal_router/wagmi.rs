use crate::{ProviderType, SwapperProvider, uniswap::v3::UniversalRouterProvider};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{V3Deployment, get_wagmi_router_deployment_by_chain},
};
use primitives::Chain;

#[derive(Debug)]
pub struct WagmiUniversalRouter {
    pub provider: ProviderType,
}

impl Default for WagmiUniversalRouter {
    fn default() -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Wagmi),
        }
    }
}

impl UniversalRouterProvider for WagmiUniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::FiveHundred, FeeTier::ThousandFiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_wagmi_router_deployment_by_chain(chain)
    }
}
