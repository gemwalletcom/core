pub type SwapperError = swapper::SwapperError;

#[uniffi::remote(Enum)]
pub enum SwapperError {
    NotSupportedChain,
    NotSupportedAsset,
    NotSupportedPair,
    NoAvailableProvider,
    InvalidAddress(String),
    InvalidAmount(String),
    InputAmountTooSmall,
    InvalidRoute,
    NetworkError(String),
    ABIError(String),
    ComputeQuoteError(String),
    TransactionError(String),
    NoQuoteAvailable,
    NotImplemented,
}
