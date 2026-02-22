mod alien;
mod approval;
mod chainlink;
mod eth_address;
mod swapper_trait;

#[cfg(test)]
pub mod testkit;

pub mod across;
pub mod asset;
pub mod chainflip;
pub mod client_factory;
pub mod config;
pub mod error;
pub mod hyperliquid;
pub mod jupiter;
pub mod models;
pub mod near_intents;
pub mod permit2_data;
pub mod proxy;
pub mod slippage;
pub mod solana;
pub mod swapper;
pub mod thorchain;
pub mod uniswap;

use number_formatter::BigNumberFormatter;

/// Converts a human-readable amount string to base units value.
pub fn amount_to_value(token: &str, decimals: u32) -> Option<String> {
    let cleaned = token.replace([',', '_'], "");
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.contains('.') {
        BigNumberFormatter::value_from_amount(&cleaned, decimals).ok()
    } else {
        Some(cleaned)
    }
}

#[cfg(feature = "reqwest_provider")]
pub use alien::reqwest_provider::NativeProvider;
pub use alien::{AlienError, HttpMethod, RpcClient, RpcProvider, Target};
pub use error::SwapperError;
pub use models::*;
pub(crate) use swapper_trait::Swapper;

pub type SwapperProvider = primitives::SwapProvider;
pub type SwapperProviderMode = primitives::swap::SwapProviderMode;
pub type SwapperQuoteAsset = primitives::swap::QuoteAsset;
pub type SwapperMode = primitives::swap::SwapMode;
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
pub type SwapperSwapStatus = primitives::swap::SwapStatus;
