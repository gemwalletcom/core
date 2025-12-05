pub type SwapperError = swapper::SwapperError;

#[uniffi::remote(Enum)]
pub enum SwapperError {
    NotSupportedChain,
    NotSupportedAsset,
    NoAvailableProvider,
    InputAmountTooSmall,
    InvalidRoute,
    ComputeQuoteError(String),
    TransactionError(String),
    NoQuoteAvailable,
}
