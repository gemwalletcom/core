pub mod error;
pub mod mime;
pub mod mock;
pub mod provider;
#[cfg(feature = "reqwest_provider")]
pub mod reqwest_provider;
pub mod signer;
pub mod target;

pub use error::AlienError;
pub use provider::AlienProvider;
pub use signer::AlienSigner;
pub use target::{AlienHttpMethod, AlienTarget, X_CACHE_TTL};

use primitives::Chain;
use std::str::FromStr;

uniffi::custom_type!(Chain, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| Chain::from_str(&s).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid Chain")),
});
