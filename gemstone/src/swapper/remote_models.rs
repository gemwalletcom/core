use primitives::AssetId;

pub type GemSwapProvider = primitives::SwapProvider;
pub type GemSwapProviderMode = swap_primitives::SwapProviderMode;
pub type GemQuoteAsset = swap_primitives::QuoteAsset;
pub type GemSwapMode = swap_primitives::SwapMode;
pub type GemSlippage = swap_primitives::Slippage;
pub type GemSlippageMode = swap_primitives::SlippageMode;

#[uniffi::remote(Enum)]
pub enum GemSwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeSwapV3,
    PancakeSwapAptosV2,
    Thorchain,
    Orca,
    Jupiter,
    Across,
    Oku,
    Wagmi,
    Cetus,
    StonFiV2,
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
    pub id: AssetId,
    pub asset_id: String,
    pub symbol: String,
    pub decimals: u32,
}
