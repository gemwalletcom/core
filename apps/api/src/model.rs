use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, Clone, AsRefStr, EnumString, EnumIter, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum APIService {
    Api,
    WebsocketPrices,
}
