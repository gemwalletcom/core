pub mod approval;
pub mod mode;
pub mod slippage;
use serde::{Deserialize, Serialize};
pub use approval::*;
pub use mode::*;
pub use slippage::*;

pub use crate::swap::approval::SwapQuoteData as QuoteData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub quote: QuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
    pub eta_in_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub from_address: String,
    pub to_address: String,
    pub from_asset: QuoteAsset,
    pub to_asset: QuoteAsset,
    pub from_value: String,
    pub referral_bps: u32,
    pub slippage_bps: u32,
}
