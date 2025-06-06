#[cfg(feature = "rpc")]
pub mod rpc;

// Taken from https://github.com/ston-fi/tonlib-rs, it should be modular crate with feature flags
pub mod address;
pub mod cell;
