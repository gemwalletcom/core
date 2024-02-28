use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum EVMChain {
    Ethereum,
    SmartChain,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    AvalancheC,
    OpBNB,
    Fantom,
    Gnosis,
    Manta,
}

impl EVMChain {
    pub fn all() -> Vec<EVMChain> {
        EVMChain::iter().collect::<Vec<_>>()
    }
}
