#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub use rpc::client::TonClient;

// Taken from https://github.com/ston-fi/tonlib-rs, it should be modular crate with feature flags
pub mod address;
pub mod cell;
