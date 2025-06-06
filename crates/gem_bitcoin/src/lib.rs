#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub use rpc::client::BitcoinClient;
