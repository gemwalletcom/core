pub mod asset;
pub mod block_explorer;
pub mod bsc;
pub mod chain;
pub mod config;
pub mod cosmos;
pub mod network;
pub mod payment;
pub mod solana;
pub mod sui;
pub mod swapper;
pub mod ton;
pub mod tron;
pub mod wallet_connect;

use network::AlienError;

uniffi::setup_scaffolding!("gemstone");
static LIB_VERSION: &str = env!("CARGO_PKG_VERSION");

#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

#[uniffi::export]
pub fn lib_version() -> String {
    String::from(LIB_VERSION)
}

/// GemstoneError
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GemstoneError {
    #[error("{msg}")]
    AnyError { msg: String },
}

impl From<anyhow::Error> for GemstoneError {
    fn from(error: anyhow::Error) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<Box<dyn std::error::Error>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<AlienError> for GemstoneError {
    fn from(error: AlienError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}
