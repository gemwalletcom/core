// gem_tron/src/lib.rs

pub mod address;

// RPC module, feature-gated
#[cfg(feature = "rpc")]
pub mod rpc;

pub mod models;
