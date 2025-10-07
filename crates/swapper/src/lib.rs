mod alien;
mod approval;
mod chainlink;
mod eth_address;
mod permit2_data;
mod sui;
mod swap_config;
mod swapper_trait;
mod tron;

#[cfg(test)]
pub mod testkit;

pub mod across;
pub mod asset;
pub mod cetus;
pub mod chainflip;
pub mod error;
pub mod gem_swapper;
pub mod hyperliquid;
pub mod jupiter;
pub mod models;
pub mod pancakeswap_aptos;
pub mod proxy;
pub mod slippage;
pub mod thorchain;
pub mod uniswap;

pub use alien::{AlienClient, AlienError, AlienProvider, AlienTarget, jsonrpc_client_with_chain};
#[cfg(feature = "reqwest_provider")]
pub use alien::reqwest_provider::NativeProvider;
pub use error::*;
pub use models::*;
pub use swapper_trait::*;
pub use swap_config::{get_default_slippage, get_swap_config, SwapConfig, SwapReferralFee, SwapReferralFees, DEFAULT_CHAINFLIP_FEE_BPS,
    DEFAULT_SLIPPAGE_BPS, DEFAULT_STABLE_SWAP_REFERRAL_BPS, DEFAULT_SWAP_FEE_BPS};

pub type SwapperProvider = primitives::SwapProvider;
pub type SwapperProviderMode = primitives::swap::SwapProviderMode;
pub type SwapperQuoteAsset = primitives::swap::QuoteAsset;
pub type SwapperMode = primitives::swap::SwapMode;
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
pub type SwapperSwapStatus = primitives::swap::SwapStatus;
