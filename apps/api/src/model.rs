use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, Clone, AsRefStr, EnumString, EnumIter, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum APIService {
    Api,
    WebsocketPrices,
}
