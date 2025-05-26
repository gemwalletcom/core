//! # gem_xrp
//!
//! This crate provides XRP Ledger specific functionalities for Gem Wallet.
//! It includes modules for XRP models and rpc operations, activated by feature flags.

#[cfg(feature = "models")]
pub mod models;

#[cfg(feature = "rpc")]
pub mod rpc;
