pub type GemSwapProvider = primitives::SwapProvider;
pub type GemSwapProviderMode = primitives::swap::SwapProviderMode;
pub type GemQuoteAsset = primitives::swap::QuoteAsset;
pub type GemSwapMode = primitives::swap::SwapMode;
pub type GemSlippage = primitives::swap::Slippage;
pub type GemSlippageMode = primitives::swap::SlippageMode;

#[uniffi::remote(Enum)]
pub enum GemSwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
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
}

#[uniffi::remote(Enum)]
pub enum GemSwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
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
