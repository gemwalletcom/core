pub mod approval;
pub mod mode;
pub mod price_impact;
pub mod quote_asset;
pub mod slippage;
pub use approval::SwapQuoteData;
pub use approval::*;
pub use mode::*;
pub use price_impact::*;
pub use quote_asset::QuoteAsset;
pub mod result;
pub mod result_mapper;
pub use result::*;
pub use result_mapper::map_swap_result;
use serde::{Deserialize, Serialize};
pub use slippage::*;
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ProxyQuote {
    pub quote: ProxyQuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
    pub eta_in_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct ProxyQuoteRequest {
    pub from_address: String,
    pub to_address: String,
    pub from_asset: QuoteAsset,
    pub to_asset: QuoteAsset,
    pub from_value: String,
    pub referral_bps: u32,
    pub slippage_bps: u32,
    pub use_max_amount: bool,
}
