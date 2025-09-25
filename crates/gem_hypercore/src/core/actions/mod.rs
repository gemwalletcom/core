pub mod agent;
pub mod user;

pub use agent::*;
pub use user::*;

pub const MAINNET: &str = "Mainnet";
pub const SIGNATURE_CHAIN_ID: &str = "0xa4b1";

pub const HYPERCORE_SIGNATURE_CHAIN_ID: &str = "0x3e7";
pub const SLIPPAGE_BUFFER_PERCENT: f64 = 0.08;
