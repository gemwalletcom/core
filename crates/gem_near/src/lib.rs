#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub use rpc::client::NearClient;
// Add other re-exports like NearMapper if needed under the rpc feature
