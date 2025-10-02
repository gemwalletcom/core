use super::{
    universal_router::{self, UniversalRouterProvider},
    v3::UniswapV3,
    v4::UniswapV4,
};
use crate::network::{AlienClient, AlienEvmRpcFactory, AlienProvider};
use std::sync::Arc;

impl UniswapV3<AlienClient, AlienEvmRpcFactory> {
    pub fn new_with_provider(provider: Box<dyn UniversalRouterProvider>, rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
        Self::new(provider, factory)
    }
}

impl UniswapV4<AlienClient, AlienEvmRpcFactory> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
        Self::with_factory(factory)
    }
}

pub fn new_uniswap_v3() -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_uniswap_v3_with_factory()
}

pub fn new_pancakeswap() -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_pancakeswap_with_factory()
}

pub fn new_aerodrome() -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_aerodrome_with_factory()
}

pub fn new_oku() -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_oku_with_factory()
}

pub fn new_wagmi() -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_wagmi_with_factory()
}

pub fn new_uniswap_v4() -> UniswapV4<AlienClient, AlienEvmRpcFactory> {
    universal_router::new_uniswap_v4_with_factory()
}
