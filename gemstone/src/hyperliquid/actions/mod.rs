pub mod approve_agent;
pub mod approve_builder_fee;
pub mod cancel_order;
pub mod place_order;
pub mod set_referrer;
pub mod withdrawal;

pub use approve_agent::*;
pub use approve_builder_fee::*;
pub use cancel_order::*;
pub use place_order::*;
pub use set_referrer::*;
pub use withdrawal::*;

pub const MAINNET: &str = "Mainnet";
pub const SIGNATURE_CHAIN_ID: &str = "0xa4b1";
