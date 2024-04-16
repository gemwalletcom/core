use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CosmosChain {
    Cosmos,
    Osmosis,
    Celestia,
    Thorchain,
    Injective,
    Sei,
    Noble,
    Dymension,
    Saga,
}

#[allow(non_camel_case_types)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum CosmosDenom {
    rune,
    uatom,
    uosmo,
    utia,
    inj,
    usei,
    uusdc,
    adym,
    usaga,
}

impl CosmosChain {
    pub fn hrp(&self) -> &str {
        match self {
            Self::Cosmos => "cosmos",
            Self::Osmosis => "osmo",
            Self::Celestia => "celestia",
            Self::Noble => "noble",
            Self::Injective => "inj",
            Self::Sei => "sei",
            Self::Thorchain => "thor",
            Self::Dymension => "dym",
            Self::Saga => "saga",
        }
    }
}
