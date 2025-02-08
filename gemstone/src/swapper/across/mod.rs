pub mod provider;
pub use provider::Across;
pub mod api;
pub mod config_store;
pub mod hubpool;

const DEFAULT_FILL_TIMEOUT: u32 = 60 * 60 * 6; // 6 hours
const DEFAULT_DEPOSIT_GAS_LIMIT: u64 = 90_000; // for both ETH and ERC20 token
const DEFAULT_FILL_GAS_LIMIT: u64 = 120_000; // for both ETH and ERC20 token
