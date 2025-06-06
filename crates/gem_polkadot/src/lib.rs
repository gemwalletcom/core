#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub use rpc::client::PolkadotClient;
// Add other re-exports like PolkadotMapper if needed under the rpc feature
