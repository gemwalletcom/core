pub mod provider;
pub use provider::Across;
pub mod api;
pub mod config_store;
pub mod hubpool;

const DEFAULT_FILL_TIMEOUT: u32 = 60 * 60 * 6; // 6 hours
const GEM_IDENTIFIER: [u8; 5] = [0x1d, 0xc0, 0xde, 0x00, 0x60];
