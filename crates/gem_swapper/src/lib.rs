mod alien;
mod approval;
mod chainlink;
mod eth_address;
mod permit2_data;
mod swapper_trait;

#[cfg(test)]
pub mod testkit;

pub mod across;
pub mod asset;
pub mod cetus;
pub mod chainflip;
pub mod client_factory;
pub mod config;
pub mod error;
pub mod hyperliquid;
pub mod jupiter;
pub mod models;
pub mod pancakeswap_aptos;
pub mod proxy;
pub mod slippage;
pub mod swapper;
pub mod thorchain;
pub mod uniswap;

#[cfg(feature = "reqwest_provider")]
pub use alien::reqwest_provider::NativeProvider;
pub use alien::{AlienError, RpcClient, RpcProvider, Target};
pub use error::*;
pub use models::*;
pub use swapper_trait::*;

pub type SwapperProvider = primitives::SwapProvider;
pub type SwapperProviderMode = primitives::swap::SwapProviderMode;
pub type SwapperQuoteAsset = primitives::swap::QuoteAsset;
pub type SwapperMode = primitives::swap::SwapMode;
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
pub type SwapperSwapStatus = primitives::swap::SwapStatus;
