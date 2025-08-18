pub mod types;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub use client::*;

// Default client type based on features
#[cfg(feature = "reqwest")]
pub type DefaultJsonRpcClient = client::JsonRpcClient<gem_client::ReqwestClient>;

// Legacy alias for backward compatibility
#[cfg(feature = "reqwest")]
pub type JsonRpcClient = DefaultJsonRpcClient;
