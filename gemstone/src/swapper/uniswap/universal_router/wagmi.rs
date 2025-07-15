use crate::swapper::{uniswap::v3::UniversalRouterProvider, SwapperProvider, SwapperProviderType};
use gem_evm::uniswap::{
    deployment::v3::{get_wagmi_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct WagmiUniversalRouter {
    pub provider: SwapperProviderType,
}

impl Default for WagmiUniversalRouter {
    fn default() -> Self {
        Self {
            provider: SwapperProviderType::new(SwapperProvider::Wagmi),
        }
    }
}

impl UniversalRouterProvider for WagmiUniversalRouter {
    fn provider(&self) -> &SwapperProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::FiveHundred, FeeTier::ThousandFiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_wagmi_router_deployment_by_chain(chain)
    }
}
