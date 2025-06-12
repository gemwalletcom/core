pub mod error;
pub mod jsonrpc;
pub mod mime;
pub mod mock;
pub mod provider;
pub mod target;

pub use error::AlienError;
pub use provider::AlienProvider;
pub use target::{AlienHttpMethod, AlienTarget};

#[cfg(feature = "reqwest_provider")]
pub mod reqwest_provider;
#[cfg(feature = "reqwest_provider")]
pub use reqwest_provider::NativeProvider;

use primitives::Chain;
use std::str::FromStr;

uniffi::custom_type!(Chain, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| Chain::from_str(&s).map_err(|_| anyhow::anyhow!("Invalid Chain")),
});
