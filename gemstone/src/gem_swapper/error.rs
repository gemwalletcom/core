pub type SwapperError = swapper::SwapperError;

#[uniffi::remote(Enum)]
pub enum SwapperError {
    NotSupportedChain,
    NotSupportedAsset,
    NoAvailableProvider,
    InputAmountError { min_amount: Option<String> },
    InvalidRoute,
    ComputeQuoteError(String),
    TransactionError(String),
    NoQuoteAvailable,
}
