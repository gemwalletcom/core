mod client;
mod default;
mod model;
mod provider;
pub use provider::Jupiter;

pub const PROGRAM_ADDRESS: &str = gem_solana::JUPITER_PROGRAM_ID;
