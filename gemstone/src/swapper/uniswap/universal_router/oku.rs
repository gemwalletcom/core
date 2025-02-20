use crate::swapper::{uniswap::v3::UniversalRouterProvider, SwapProvider};
use gem_evm::uniswap::{
    deployment::v3::{get_oku_deployment_by_chain, V3Deployment},
    FeeTier,
};
use primitives::Chain;

#[derive(Debug)]
pub struct OkuUniversalRouter {}

impl UniversalRouterProvider for OkuUniversalRouter {
    fn provider(&self) -> SwapProvider {
        SwapProvider::OkuTrade
    }

    fn get_tiers(&self) -> Vec<gem_evm::uniswap::FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        get_oku_deployment_by_chain(chain)
    }
}
