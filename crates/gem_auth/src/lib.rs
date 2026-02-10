#[cfg(feature = "client")]
mod client;
mod device_signature;
#[cfg(feature = "client")]
mod jwt;
mod signature;

#[cfg(feature = "client")]
pub use client::AuthClient;
pub use device_signature::verify_device_signature;
#[cfg(feature = "client")]
pub use jwt::{JwtClaims, create_device_token, verify_device_token};
pub use signature::{AuthMessageData, create_auth_hash, verify_auth_signature};
