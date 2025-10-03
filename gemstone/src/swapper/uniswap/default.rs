use super::{universal_router, v3::UniswapV3, v4::UniswapV4};
use crate::{
    network::{AlienClient, AlienEvmRpcFactory, AlienProvider},
    swapper::Swapper,
};
use std::sync::Arc;

fn factory_from_provider(rpc_provider: Arc<dyn AlienProvider>) -> Arc<AlienEvmRpcFactory> {
    Arc::new(AlienEvmRpcFactory::new(rpc_provider))
}

pub fn new_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    universal_router::new_uniswap_v3_with_factory::<AlienClient, AlienEvmRpcFactory>(factory)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    universal_router::new_pancakeswap_with_factory::<AlienClient, AlienEvmRpcFactory>(factory)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    universal_router::new_aerodrome_with_factory::<AlienClient, AlienEvmRpcFactory>(factory)
}

pub fn new_oku(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    universal_router::new_oku_with_factory::<AlienClient, AlienEvmRpcFactory>(factory)
}

pub fn new_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    universal_router::new_wagmi_with_factory::<AlienClient, AlienEvmRpcFactory>(factory)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV4<AlienClient, AlienEvmRpcFactory> {
    let factory = factory_from_provider(rpc_provider);
    UniswapV4::with_factory(factory)
}

pub fn boxed_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_uniswap_v3(rpc_provider))
}

pub fn boxed_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_pancakeswap(rpc_provider))
}

pub fn boxed_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_aerodrome(rpc_provider))
}

pub fn boxed_oku(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_oku(rpc_provider))
}

pub fn boxed_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_wagmi(rpc_provider))
}

pub fn boxed_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Box::new(new_uniswap_v4(rpc_provider))
}
