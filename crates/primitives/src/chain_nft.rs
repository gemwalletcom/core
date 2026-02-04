use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::Chain;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NFTChain {
    Ethereum,
    Polygon,
    Solana,
    SmartChain,
}

impl NFTChain {
    pub fn all() -> Vec<NFTChain> {
        NFTChain::iter().collect()
    }
}

impl From<NFTChain> for Chain {
    fn from(chain: NFTChain) -> Self {
        match chain {
            NFTChain::Ethereum => Chain::Ethereum,
            NFTChain::Polygon => Chain::Polygon,
            NFTChain::Solana => Chain::Solana,
            NFTChain::SmartChain => Chain::SmartChain,
        }
    }
}
