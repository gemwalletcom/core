use super::{
    universal_router,
    v3::UniswapV3,
    v4::UniswapV4,
};
use crate::network::{AlienClient, AlienEvmRpcFactory, AlienProvider};
use std::sync::Arc;

pub fn new_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_uniswap_v3_with_factory(factory)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_pancakeswap_with_factory(factory)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_aerodrome_with_factory(factory)
}

pub fn new_oku(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_oku_with_factory(factory)
}

pub fn new_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_wagmi_with_factory(factory)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV4<AlienClient, AlienEvmRpcFactory> {
    let factory = Arc::new(AlienEvmRpcFactory::new(rpc_provider));
    universal_router::new_uniswap_v4_with_factory(factory)
}
