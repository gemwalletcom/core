use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

use crate::Chain;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
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
}

impl CosmosChain {
    pub fn from_chain(chain: Chain) -> Option<Self> {
        CosmosChain::from_str(chain.as_ref()).ok()
    }

    pub fn as_chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }

    pub fn hrp(&self) -> &str {
        match self {
            Self::Cosmos => "cosmos",
            Self::Osmosis => "osmo",
            Self::Celestia => "celestia",
            Self::Thorchain => "thor",
            Self::Injective => "inj",
            Self::Sei => "sei",
            Self::Noble => "noble",
        }
    }

    pub fn denom(&self) -> CosmosDenom {
        match self {
            Self::Cosmos => CosmosDenom::Uatom,
            Self::Osmosis => CosmosDenom::Uosmo,
            Self::Celestia => CosmosDenom::Utia,
            Self::Thorchain => CosmosDenom::Rune,
            Self::Injective => CosmosDenom::Inj,
            Self::Sei => CosmosDenom::Usei,
            Self::Noble => CosmosDenom::Uusdc,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum CosmosDenom {
    Rune,
    Uatom,
    Uosmo,
    Utia,
    Inj,
    Usei,
    Uusdc,
}
