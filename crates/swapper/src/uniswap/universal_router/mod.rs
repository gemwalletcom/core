use crate::{
    ProviderType, SwapperProvider,
    alien::RpcProvider,
    uniswap::{
        v3::{UniswapV3, UniversalRouterProvider},
        v4::UniswapV4,
    },
};
use gem_evm::uniswap::{
    FeeTier,
    deployment::v3::{
        V3Deployment, get_aerodrome_router_deployment_by_chain, get_oku_deployment_by_chain, get_pancakeswap_router_deployment_by_chain,
        get_uniswap_router_deployment_by_chain, get_wagmi_router_deployment_by_chain,
    },
};
use primitives::Chain;
use std::sync::Arc;

type DeploymentFn = fn(&Chain) -> Option<V3Deployment>;

#[derive(Debug)]
struct UniversalRouter {
    provider: ProviderType,
    tiers: Vec<FeeTier>,
    deployment_fn: DeploymentFn,
}

impl UniversalRouter {
    fn new(id: SwapperProvider, tiers: Vec<FeeTier>, deployment_fn: DeploymentFn) -> Self {
        Self {
            provider: ProviderType::new(id),
            tiers,
            deployment_fn,
        }
    }
}

impl UniversalRouterProvider for UniversalRouter {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        self.tiers.clone()
    }

    fn get_deployment_by_chain(&self, chain: &Chain) -> Option<V3Deployment> {
        (self.deployment_fn)(chain)
    }
}

pub fn new_uniswap_v3(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV3 {
    let router = UniversalRouter::new(
        SwapperProvider::UniswapV3,
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand],
        get_uniswap_router_deployment_by_chain,
    );
    UniswapV3::new(Box::new(router), rpc_provider)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV3 {
    let router = UniversalRouter::new(
        SwapperProvider::PancakeswapV3,
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::TwoThousandFiveHundred, FeeTier::TenThousand],
        get_pancakeswap_router_deployment_by_chain,
    );
    UniswapV3::new(Box::new(router), rpc_provider)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV3 {
    let router = UniversalRouter::new(
        SwapperProvider::Aerodrome,
        vec![FeeTier::Hundred, FeeTier::FourHundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand],
        get_aerodrome_router_deployment_by_chain,
    );
    UniswapV3::new(Box::new(router), rpc_provider)
}

pub fn new_oku(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV3 {
    let router = UniversalRouter::new(
        SwapperProvider::Oku,
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand],
        get_oku_deployment_by_chain,
    );
    UniswapV3::new(Box::new(router), rpc_provider)
}

pub fn new_wagmi(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV3 {
    let router = UniversalRouter::new(
        SwapperProvider::Wagmi,
        vec![FeeTier::FiveHundred, FeeTier::ThousandFiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand],
        get_wagmi_router_deployment_by_chain,
    );
    UniswapV3::new(Box::new(router), rpc_provider)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn RpcProvider>) -> UniswapV4 {
    UniswapV4::new(rpc_provider)
}
