pub mod coingecko;
pub mod price_updater;
pub mod client;
pub mod price_mapper;
pub mod storage;
pub use crate::storage::db::ClickhouseDatabase;

pub const DEFAULT_FIAT_CURRENCY: &str = "USD";