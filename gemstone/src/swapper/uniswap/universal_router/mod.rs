mod aerodrome;
mod oku;
mod pancakeswap;
mod uniswap_v3;
mod wagmi;

use crate::{
    network::AlienProvider,
    swapper::uniswap::{v3::UniswapV3, v4::UniswapV4},
};
use std::sync::Arc;

pub fn new_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(uniswap_v3::UniswapUniversalRouter::default()), rpc_provider)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(pancakeswap::PancakeSwapUniversalRouter::default()), rpc_provider)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(aerodrome::AerodromeUniversalRouter::default()), rpc_provider)
}

pub fn new_oku(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(oku::OkuUniversalRouter::default()), rpc_provider)
}

pub fn new_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(wagmi::WagmiUniversalRouter::default()), rpc_provider)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV4 {
    UniswapV4::new(rpc_provider)
}
