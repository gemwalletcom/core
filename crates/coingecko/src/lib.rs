pub mod client;
pub mod mapper;
pub mod model;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub use self::client::{COINGECKO_API_HOST, COINGECKO_API_PRO_HOST};
pub use self::mapper::{get_chain_for_coingecko_platform_id, get_chains_for_coingecko_market_id, get_coingecko_market_id_for_chain, get_coingecko_platform_id_for_chain};
pub use self::model::{Coin, CoinGeckoErrorResponse, CoinGeckoResponse, CoinInfo, CoinMarket};

use gem_client::ReqwestClient;
pub type CoinGeckoClient = client::CoinGeckoClient<ReqwestClient>;
