#[cfg(any(feature = "rpc", feature = "signer"))]
mod address;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod cbor;
pub mod models;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod planner;
#[cfg(feature = "rpc")]
pub mod provider;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(any(feature = "rpc", feature = "signer"))]
mod transaction;

#[cfg(feature = "rpc")]
pub use provider::map_transaction;
#[cfg(feature = "rpc")]
pub use rpc::client::CardanoClient;

#[cfg(feature = "signer")]
pub mod signer;

#[cfg(any(feature = "rpc", feature = "signer"))]
pub fn validate_address(address: &str) -> bool {
    address::ShelleyAddress::parse(address).is_ok()
}
