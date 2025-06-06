//! # gem_xrp
//!
//! This crate provides XRP Ledger specific functionalities for Gem Wallet.
//! It includes modules for XRP models and rpc operations, activated by feature flags.

#[cfg(feature = "typeshare")]
pub mod typeshare;

#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "rpc")]
pub use rpc::constants::*;
