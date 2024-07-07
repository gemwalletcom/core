use crate::Chain;
use serde::{Deserialize, Serialize};
use std::vec;
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
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
    Blast,
    ZkSync,
    Linea,
    Mantle,
    Celo,
}

impl EVMChain {
    pub fn all() -> Vec<EVMChain> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        match self {
            Self::Ethereum => 1_000_000_000,   // https://etherscan.io/gastracker
            Self::SmartChain => 1_000_000_000, // https://bscscan.com/gastracker
            Self::Polygon => 30_000_000_000,   // https://polygonscan.com/gastracker
            Self::Arbitrum => 10_000_000, // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
            Self::Optimism => 10_000_000, // https://optimistic.etherscan.io/chart/gasprice
            Self::Base => 100_000_000,    // https://basescan.org/chart/gasprice
            Self::AvalancheC => 25_000_000_000, // https://snowscan.xyz/gastracker
            Self::OpBNB => 1_000_000,     // https://opbnbscan.com/statistics
            Self::Fantom => 3_500_000_000, // https://ftmscan.com/gastracker
            Self::Gnosis => 3_000_000_000, // https://gnosisscan.io/gastracker
            Self::Blast => 200_000_000,   // https://blastscan.io/chart/gasprice
            Self::ZkSync => 20_000_000,   // https://era.zksync.network/chart/gasprice
            Self::Linea => 50_000_000,    // https://lineascan.build/gastracker
            Self::Mantle | Self::Celo | Self::Manta => 10_000_000,
        }
    }

    pub fn is_opstack(&self) -> bool {
        match self {
            Self::Ethereum
            | Self::SmartChain
            | Self::Polygon
            | Self::Arbitrum
            | Self::AvalancheC
            | Self::Fantom
            | Self::Gnosis
            | Self::Manta
            | Self::Blast
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo => false,
            Self::Optimism | Self::Base | Self::OpBNB => true,
        }
    }

    pub fn swap_whitelist_contracts(&self) -> Vec<&'static str> {
        match self {
            Self::Ethereum
            | Self::SmartChain
            | Self::Polygon
            | Self::Arbitrum
            | Self::AvalancheC
            | Self::Fantom
            | Self::Gnosis
            | Self::Optimism
            | Self::Base => vec!["0x1111111254EEB25477B68fb85Ed929f73A960582"], // 1inch
            Self::ZkSync => vec!["0x6e2B76966cbD9cF4cC2Fa0D76d24d5241E0ABC2F"], // 1inch
            Self::Manta | Self::Blast | Self::Linea | Self::Mantle | Self::OpBNB | Self::Celo => {
                vec![]
            }
        }
    }

    pub fn from_chain(chain: Chain) -> Option<Self> {
        match chain {
            Chain::Ethereum => Some(Self::Ethereum),
            Chain::SmartChain => Some(Self::SmartChain),
            Chain::Polygon => Some(Self::Polygon),
            Chain::Arbitrum => Some(Self::Arbitrum),
            Chain::Optimism => Some(Self::Optimism),
            Chain::Base => Some(Self::Base),
            Chain::AvalancheC => Some(Self::AvalancheC),
            Chain::OpBNB => Some(Self::OpBNB),
            Chain::Fantom => Some(Self::Fantom),
            Chain::Gnosis => Some(Self::Gnosis),
            Chain::ZkSync => Some(Self::ZkSync),
            Chain::Linea => Some(Self::Linea),
            Chain::Manta => Some(Self::Manta),
            Chain::Celo => Some(Self::Celo),
            _ => None,
        }
    }

    pub fn to_chain(&self) -> Chain {
        match self {
            Self::Ethereum => Chain::Ethereum,
            Self::SmartChain => Chain::SmartChain,
            Self::Polygon => Chain::Polygon,
            Self::Arbitrum => Chain::Arbitrum,
            Self::Optimism => Chain::Optimism,
            Self::Base => Chain::Base,
            Self::AvalancheC => Chain::AvalancheC,
            Self::OpBNB => Chain::OpBNB,
            Self::Fantom => Chain::Fantom,
            Self::Gnosis => Chain::Gnosis,
            Self::Manta => Chain::Manta,
            Self::Blast => Chain::Blast,
            Self::ZkSync => Chain::ZkSync,
            Self::Linea => Chain::Linea,
            Self::Mantle => Chain::Mantle,
            Self::Celo => Chain::Celo,
        }
    }
}
