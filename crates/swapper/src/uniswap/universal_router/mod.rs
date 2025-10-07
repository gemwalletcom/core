mod aerodrome;
mod oku;
mod pancakeswap;
mod uniswap_v3;
mod wagmi;

use crate::{
    alien::factory::JsonRpcClientFactory,
    uniswap::{v3::UniswapV3, v4::UniswapV4},
};
use gem_client::Client;
use std::{fmt::Debug, sync::Arc};

type UniV3Router = uniswap_v3::UniswapUniversalRouter;
type PancakeRouter = pancakeswap::PancakeSwapUniversalRouter;
type AerodromeRouter = aerodrome::AerodromeUniversalRouter;
type OkuRouter = oku::OkuUniversalRouter;
type WagmiRouter = wagmi::WagmiUniversalRouter;

pub fn new_uniswap_v3_with_factory<C, F>(factory: Arc<F>) -> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV3::new(Box::new(UniV3Router::default()), factory)
}

pub fn new_pancakeswap_with_factory<C, F>(factory: Arc<F>) -> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV3::new(Box::new(PancakeRouter::default()), factory)
}

pub fn new_aerodrome_with_factory<C, F>(factory: Arc<F>) -> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV3::new(Box::new(AerodromeRouter::default()), factory)
}

pub fn new_oku_with_factory<C, F>(factory: Arc<F>) -> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV3::new(Box::new(OkuRouter::default()), factory)
}

pub fn new_wagmi_with_factory<C, F>(factory: Arc<F>) -> UniswapV3<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV3::new(Box::new(WagmiRouter::default()), factory)
}

pub fn new_uniswap_v4_with_factory<C, F>(factory: Arc<F>) -> UniswapV4<C, F>
where
    C: Client + Clone + Debug + Send + Sync + 'static,
    F: JsonRpcClientFactory<C>,
{
    UniswapV4::with_factory(factory)
}
