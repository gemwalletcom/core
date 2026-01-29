#[cfg(feature = "client")]
mod client;
mod device_signature;
mod signature;

#[cfg(feature = "client")]
pub use client::AuthClient;
pub use device_signature::verify_device_signature;
pub use signature::{AuthMessageData, create_auth_hash, verify_auth_signature};
