use gem_evm::uniswap::{
    deployment::{get_aerodrome_router_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

use super::{SwapProvider, UniversalRouterProvider};

#[derive(Debug)]
pub struct AerodromeUniversalRouter {}

impl UniversalRouterProvider for AerodromeUniversalRouter {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Aerodrome
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![
            // FeeTier::Hundred,
            // FeeTier::FiveHundred,
            FeeTier::SixThousand,
            FeeTier::TenThousand,
            FeeTier::TwentyThousand,
        ]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_aerodrome_router_deployment_by_chain(chain)
    }
}
