mod pancakeswap;
mod uniswap_v3;

use crate::swapper::uniswap::{v3::UniswapV3, v4::UniswapV4};

pub fn new_uniswap_v3() -> UniswapV3 {
    UniswapV3::new(Box::new(uniswap_v3::UniswapUniversalRouter {}))
}

pub fn new_pancakeswap() -> UniswapV3 {
    UniswapV3::new(Box::new(pancakeswap::PancakeSwapUniversalRouter {}))
}

pub fn new_uniswap_v4() -> UniswapV4 {
    UniswapV4::default()
}
