pub mod alien;
pub mod block_explorer;
pub mod config;
pub mod ethereum;
pub mod gateway;
pub mod gem_swapper;
pub mod hyperliquid;
pub mod message;
pub mod models;
pub mod network;
pub mod signer;
pub mod payment;
pub mod sui;
pub mod wallet_connect;

use alien::AlienError;

uniffi::setup_scaffolding!("gemstone");
static LIB_VERSION: &str = env!("CARGO_PKG_VERSION");

#[uniffi::export]
pub fn lib_version() -> String {
    String::from(LIB_VERSION)
}

/// GemstoneError
#[derive(Debug, uniffi::Error)]
pub enum GemstoneError {
    AnyError { msg: String },
}

impl std::fmt::Display for GemstoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyError { msg } => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GemstoneError {}

impl From<Box<dyn std::error::Error + Send + Sync>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<String> for GemstoneError {
    fn from(error: String) -> Self {
        Self::AnyError { msg: error }
    }
}

impl From<Box<dyn std::error::Error>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<primitives::payment_decoder::PaymentDecoderError> for GemstoneError {
    fn from(error: primitives::payment_decoder::PaymentDecoderError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<AlienError> for GemstoneError {
    fn from(error: AlienError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}
