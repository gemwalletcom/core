pub mod client;
pub mod model;

pub use self::client::CoinMarketCapClient;
pub use self::model::{AltSeasonData, AltSeasonPoint, FearGreedData, FearGreedItem};
