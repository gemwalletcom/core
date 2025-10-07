use super::{universal_router, v3::UniswapV3, v4::UniswapV4};
use crate::{Swapper, alien::AlienProvider};
use std::sync::Arc;

pub fn new_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    universal_router::new_uniswap_v3(rpc_provider)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    universal_router::new_pancakeswap(rpc_provider)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    universal_router::new_aerodrome(rpc_provider)
}

pub fn new_oku(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    universal_router::new_oku(rpc_provider)
}

pub fn new_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    universal_router::new_wagmi(rpc_provider)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV4 {
    universal_router::new_uniswap_v4(rpc_provider)
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
