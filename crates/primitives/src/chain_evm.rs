use crate::Chain;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
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
            Self::Ethereum => 1_000_000_000,    // https://etherscan.io/gastracker
            Self::SmartChain => 1_000_000_000,  // https://bscscan.com/gastracker
            Self::Polygon => 30_000_000_000,    // https://polygonscan.com/gastracker
            Self::Arbitrum => 10_000_000,       // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
            Self::Optimism => 10_000_000,       // https://optimistic.etherscan.io/chart/gasprice
            Self::Base => 100_000_000,          // https://basescan.org/chart/gasprice
            Self::AvalancheC => 25_000_000_000, // https://snowscan.xyz/gastracker
            Self::OpBNB => 1_000_000,           // https://opbnbscan.com/statistics
            Self::Fantom => 3_500_000_000,      // https://ftmscan.com/gastracker
            Self::Gnosis => 3_000_000_000,      // https://gnosisscan.io/gastracker
            Self::Blast => 200_000_000,         // https://blastscan.io/chart/gasprice
            Self::ZkSync => 20_000_000,         // https://era.zksync.network/chart/gasprice
            Self::Linea => 50_000_000,          // https://lineascan.build/gastracker
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

    pub fn weth_contract(&self) -> Option<&str> {
        match self {
            Self::Ethereum => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            Self::SmartChain => Some("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"),
            Self::Polygon => Some("0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),
            Self::Arbitrum => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
            Self::Optimism | Self::Base | Self::OpBNB => Some("0x4200000000000000000000000000000000000006"),
            Self::AvalancheC => Some("0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7"),
            Self::Fantom => Some("0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83"),
            Self::Gnosis => Some("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d"),
            Self::ZkSync => Some("0x5AEa5775959fBC2557Cc8789bC1bf90A239D9a91"),
            Self::Blast => Some("0x4300000000000000000000000000000000000004"),
            Self::Celo => Some("0x471EcE3750Da237f93B8E339c536989b8978a438"),
            Self::Manta | Self::Linea | Self::Mantle => None,
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
