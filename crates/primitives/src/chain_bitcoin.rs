use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::EnumIter;
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::Chain;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum BitcoinChain {
    Bitcoin,
    BitcoinCash,
    Litecoin,
    Doge,
}

impl BitcoinChain {
    pub fn from_chain(chain: Chain) -> Option<BitcoinChain> {
        BitcoinChain::from_str(chain.as_ref()).ok()
    }
    pub fn get_chain(&self) -> Chain {
        match self {
            BitcoinChain::Bitcoin => Chain::Bitcoin,
            BitcoinChain::BitcoinCash => Chain::BitcoinCash,
            BitcoinChain::Litecoin => Chain::Litecoin,
            BitcoinChain::Doge => Chain::Doge,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BlocksFeePriority {
    pub normal: i32,
    pub slow: i32,
    pub fast: i32,
}

impl BitcoinChain {
    pub fn get_blocks_fee_priority(&self) -> BlocksFeePriority {
        match self {
            BitcoinChain::Bitcoin => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::BitcoinCash => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::Litecoin => BlocksFeePriority { slow: 6, normal: 3, fast: 1 },
            BitcoinChain::Doge => BlocksFeePriority { slow: 8, normal: 4, fast: 2 },
        }
    }
}
