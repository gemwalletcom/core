use crate::swapper::{uniswap::v3::UniversalRouterProvider, GemSwapProvider, SwapProviderType};
use gem_evm::uniswap::{
    deployment::v3::{get_aerodrome_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct AerodromeUniversalRouter {
    pub provider: SwapProviderType,
}

impl Default for AerodromeUniversalRouter {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(GemSwapProvider::Aerodrome),
        }
    }
}

impl UniversalRouterProvider for AerodromeUniversalRouter {
    fn provider(&self) -> &SwapProviderType {
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

    fn use_permit2(&self) -> bool {
        true
    }
}
