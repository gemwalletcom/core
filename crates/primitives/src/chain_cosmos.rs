use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
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
