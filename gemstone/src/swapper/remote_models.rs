use primitives::Chain;

pub type SwapperProvider = primitives::SwapProvider;
pub type SwapperProviderMode = primitives::swap::SwapProviderMode;
pub type SwapperQuoteAsset = primitives::swap::QuoteAsset;
pub type SwapperMode = primitives::swap::SwapMode;
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperApprovalData = primitives::swap::ApprovalData;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
pub type SwapperSwapStatus = primitives::swap::SwapStatus;

#[uniffi::remote(Record)]
pub struct SwapperQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<SwapperApprovalData>,
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
    NearIntents,
}

#[uniffi::remote(Record)]
pub struct SwapperApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
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
