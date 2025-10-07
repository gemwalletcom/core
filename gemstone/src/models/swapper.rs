use primitives::{Chain, swap::ApprovalData as GemApprovalData};

use gem_swapper::{
    SwapperMode, SwapperProvider, SwapperProviderMode, SwapperQuoteAsset, SwapperQuoteData, SwapperSlippage, SwapperSlippageMode, SwapperSwapStatus,
};

#[uniffi::remote(Record)]
pub struct SwapperQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum SwapperProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    Aerodrome,
    PancakeswapAptosV2,
    Thorchain,
    Jupiter,
    Across,
    Oku,
    Wagmi,
    Cetus,
    StonfiV2,
    Mayan,
    Reservoir,
    Symbiosis,
    Chainflip,
    CetusAggregator,
    Relay,
    Hyperliquid,
}

#[uniffi::remote(Enum)]
pub enum SwapperProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>),
}

#[uniffi::remote(Enum)]
pub enum SwapperMode {
    ExactIn,
    ExactOut,
}

#[uniffi::remote(Record)]
pub struct SwapperSlippage {
    pub bps: u32,
    pub mode: SwapperSlippageMode,
}

#[uniffi::remote(Enum)]
pub enum SwapperSlippageMode {
    Auto,
    Exact,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}

#[uniffi::remote(Enum)]
pub enum SwapperSwapStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}
