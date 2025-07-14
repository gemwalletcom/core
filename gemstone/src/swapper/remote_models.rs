use primitives::Chain;

pub type GemSwapProvider = primitives::SwapProvider;
pub type GemSwapProviderMode = primitives::swap::SwapProviderMode;
pub type GemQuoteAsset = primitives::swap::QuoteAsset;
pub type GemSwapMode = primitives::swap::SwapMode;
pub type GemSlippage = primitives::swap::Slippage;
pub type GemSlippageMode = primitives::swap::SlippageMode;
pub type GemApprovalData = primitives::swap::ApprovalData;
pub type GemSwapQuoteData = primitives::swap::QuoteData;

#[uniffi::remote(Record)]
pub struct GemSwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemSwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    Aerodrome,
    PancakeswapAptosV2,
    Thorchain,
    Orca,
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
}

#[uniffi::remote(Record)]
pub struct GemApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[uniffi::remote(Enum)]
pub enum GemSwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>),
}

#[uniffi::remote(Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[uniffi::remote(Record)]
pub struct GemSlippage {
    pub bps: u32,
    pub mode: GemSlippageMode,
}

#[uniffi::remote(Enum)]
pub enum GemSlippageMode {
    Auto,
    Exact,
}

#[uniffi::remote(Record)]
pub struct GemQuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}
