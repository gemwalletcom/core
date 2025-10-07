mod aerodrome;
mod oku;
mod pancakeswap;
mod uniswap_v3;
mod wagmi;

use crate::{
    alien::AlienProvider,
    uniswap::{v3::UniswapV3, v4::UniswapV4},
};
use std::sync::Arc;

type UniV3Router = uniswap_v3::UniswapUniversalRouter;
type PancakeRouter = pancakeswap::PancakeSwapUniversalRouter;
type AerodromeRouter = aerodrome::AerodromeUniversalRouter;
type OkuRouter = oku::OkuUniversalRouter;
type WagmiRouter = wagmi::WagmiUniversalRouter;

pub fn new_uniswap_v3(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(UniV3Router::default()), rpc_provider)
}

pub fn new_pancakeswap(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(PancakeRouter::default()), rpc_provider)
}

pub fn new_aerodrome(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(AerodromeRouter::default()), rpc_provider)
}

pub fn new_oku(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(OkuRouter::default()), rpc_provider)
}

pub fn new_wagmi(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV3 {
    UniswapV3::new(Box::new(WagmiRouter::default()), rpc_provider)
}

pub fn new_uniswap_v4(rpc_provider: Arc<dyn AlienProvider>) -> UniswapV4 {
    UniswapV4::new(rpc_provider)
}
