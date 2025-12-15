#[cfg(feature = "client")]
mod client;
mod signature;

#[cfg(feature = "client")]
pub use client::AuthClient;
pub use signature::{AuthMessageData, create_auth_hash, verify_auth_signature};
