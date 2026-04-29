pub const STAKING_CONTRACT: &str = "EQCkWxfyhAkim3g2DjKQQg8T5P4g-Q1-K_jErGcDJZ4i-vqR";
pub const TS_TON_MASTER: &str = "0:BDF3FA8098D129B54B4F73B5BAC5D1E1FD91EB054169C3916DFC8CCD536D1000";

#[cfg(feature = "rpc")]
mod pool;

#[cfg(feature = "rpc")]
pub use pool::{PoolFullData, get_pool_full_data};

#[cfg(feature = "signer")]
mod payload;

#[cfg(feature = "signer")]
pub use payload::{build_stake_payload_base64, build_unstake_payload_base64};

#[cfg(feature = "signer")]
pub(crate) use payload::attached_value;
