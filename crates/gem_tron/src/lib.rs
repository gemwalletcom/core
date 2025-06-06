// gem_tron/src/lib.rs

// Core Tron logic modules
pub mod abi;
pub mod address;

// RPC module, feature-gated
#[cfg(feature = "rpc")]
pub mod rpc;

// Re-export client from rpc module, feature-gated
#[cfg(feature = "rpc")]
pub use rpc::client::TronClient;
