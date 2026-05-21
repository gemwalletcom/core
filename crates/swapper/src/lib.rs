mod alien;
mod approval;
mod cache;
mod chainlink;
pub mod cross_chain;
mod eth_address;
mod fee_token;
pub mod fees;
mod swapper_trait;

#[cfg(test)]
pub mod testkit;

pub mod across;
pub mod cetus_clmm;
pub mod chainflip;
pub mod client_factory;
pub mod config;
pub mod error;
pub mod hyperliquid;
pub mod jupiter;
pub mod mayan;
pub mod models;
pub mod near_intents;
pub mod okx;
pub mod panora;
pub mod permit2_data;
pub mod proxy;
pub mod relay;
mod route_cache;
mod solana;
pub mod squid;
pub mod stonfi;
pub mod swapper;
pub mod thorchain;
pub mod uniswap;

use number_formatter::BigNumberFormatter;

pub(crate) use cache::{STATIC_READ_CACHE_TTL_SECONDS, cache_headers, static_read_cache_headers};

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
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
pub type SwapperSwapStatus = primitives::swap::SwapStatus;
pub type SwapperTransactionSwapMetadata = primitives::TransactionSwapMetadata;
pub type SwapperSwapResult = primitives::swap::SwapResult;
