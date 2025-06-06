#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub use rpc::client::StellarClient;
// Add other re-exports like StellarMapper if needed under the rpc feature
