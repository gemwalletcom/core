use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize, Clone, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Sendable, Hashable, Equatable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LinkType {
    X,
    Discord,
    Reddit,
    Telegram,
    GitHub,
    YouTube,
    Facebook,
    Website,
    Coingecko,
    OpenSea,
    Instagram,
    MagicEden,
    CoinMarketCap,
}

impl LinkType {
    pub fn name(&self) -> String {
        self.as_ref().to_string()
    }
}

impl LinkType {
    pub fn all() -> Vec<LinkType> {
        LinkType::iter().collect::<Vec<_>>()
    }
}
