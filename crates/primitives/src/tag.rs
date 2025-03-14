use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AssetTag {
    Trending,
    Gainers,
    Losers,
    New,
    Stablecoins,
}

impl AssetTag {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}
