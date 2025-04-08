mod oku;
mod pancakeswap;
mod reservoir;
mod uniswap_v3;
mod wagmi;

use crate::swapper::uniswap::{v3::UniswapV3, v4::UniswapV4};

pub fn new_uniswap_v3() -> UniswapV3 {
    UniswapV3::new(Box::new(uniswap_v3::UniswapUniversalRouter::default()))
}

pub fn new_pancakeswap() -> UniswapV3 {
    UniswapV3::new(Box::new(pancakeswap::PancakeSwapUniversalRouter::default()))
}

pub fn new_oku() -> UniswapV3 {
    UniswapV3::new(Box::new(oku::OkuUniversalRouter::default()))
}

pub fn new_wagmi() -> UniswapV3 {
    UniswapV3::new(Box::new(wagmi::WagmiUniversalRouter::default()))
}

pub fn new_reservoir() -> UniswapV3 {
    UniswapV3::new(Box::new(reservoir::ReservoirUniversalRouter::default()))
}

pub fn new_uniswap_v4() -> UniswapV4 {
    UniswapV4::default()
}
