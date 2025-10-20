use primitives::swap::SwapQuoteDataType;
pub use primitives::swap::{ApprovalData, SwapData, SwapProviderData, SwapQuote, SwapQuoteData};
pub use swapper::SwapperProvider;

pub type GemApprovalData = ApprovalData;
pub type GemSwapData = SwapData;
pub type GemSwapProviderData = SwapProviderData;
pub type GemSwapQuote = SwapQuote;
pub type GemSwapQuoteData = SwapQuoteData;
pub type GemSwapQuoteDataType = SwapQuoteDataType;

#[uniffi::remote(Record)]
pub struct GemApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[uniffi::remote(Enum)]
pub enum GemSwapQuoteDataType {
    Contract,
    Transfer,
}

#[uniffi::remote(Record)]
pub struct GemSwapData {
    pub quote: GemSwapQuote,
    pub data: GemSwapQuoteData,
}

#[uniffi::remote(Record)]
pub struct GemSwapQuote {
    pub from_address: String,
    pub from_value: String,
    pub to_address: String,
    pub to_value: String,
    pub provider_data: GemSwapProviderData,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
    pub use_max_amount: Option<bool>,
}

#[uniffi::remote(Record)]
pub struct GemSwapQuoteData {
    pub to: String,
    pub data_type: GemSwapQuoteDataType,
    pub value: String,
    pub data: String,
    pub memo: Option<String>,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Record)]
pub struct GemSwapProviderData {
    pub provider: SwapperProvider,
    pub name: String,
    pub protocol_name: String,
}
