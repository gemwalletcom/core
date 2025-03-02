use crate::network::{jsonrpc::JsonRpcError, AlienError};
use gem_evm::address::AddressError;
use std::fmt::Debug;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Not supported asset")]
    NotSupportedAsset,
    #[error("Not supported pair")]
    NotSupportedPair,
    #[error("No available provider")]
    NoAvailableProvider,
    #[error("Invalid address {address}")]
    InvalidAddress { address: String },
    #[error("Invalid input amount")]
    InvalidAmount,
    #[error("Input amount is too small")]
    InputAmountTooSmall,
    #[error("Invalid route")]
    InvalidRoute,
    #[error("RPC error: {msg}")]
    NetworkError { msg: String },
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
    #[error("Compute quote error: {msg}")]
    ComputeQuoteError { msg: String },

    #[error("No quote available")]
    NoQuoteAvailable,
    #[error("Not implemented")]
    NotImplemented,
}

impl From<AlienError> for SwapperError {
    fn from(err: AlienError) -> Self {
        Self::NetworkError { msg: err.to_string() }
    }
}

impl From<JsonRpcError> for SwapperError {
    fn from(err: JsonRpcError) -> Self {
        Self::NetworkError { msg: err.to_string() }
    }
}

impl From<AddressError> for SwapperError {
    fn from(err: AddressError) -> Self {
        Self::InvalidAddress { address: err.to_string() }
    }
}
