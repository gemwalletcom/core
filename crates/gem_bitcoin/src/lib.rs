pub mod models;

#[cfg(feature = "rpc")]
pub mod provider;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "signer")]
pub mod signer;

#[cfg(test)]
pub mod testkit;

#[cfg(feature = "rpc")]
pub use provider::map_transaction;

#[cfg(feature = "rpc")]
pub use rpc::client::BitcoinClient;
