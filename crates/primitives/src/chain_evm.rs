use crate::Chain;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq)]
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
    World,
}

impl EVMChain {
    pub fn all() -> Vec<EVMChain> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        match self {
            Self::Ethereum => 1_000_000_000,        // https://etherscan.io/gastracker
            Self::SmartChain => 1_000_000_000,      // https://bscscan.com/gastracker
            Self::Polygon => 30_000_000_000,        // https://polygonscan.com/gastracker
            Self::Arbitrum => 10_000_000,           // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
            Self::Optimism => 10_000_000,           // https://optimistic.etherscan.io/chart/gasprice
            Self::Base => 100_000_000,              // https://basescan.org/chart/gasprice
            Self::AvalancheC => 25_000_000_000,     // https://snowscan.xyz/gastracker
            Self::OpBNB | Self::World => 1_000_000, // https://opbnbscan.com/statistics
            Self::Fantom => 3_500_000_000,          // https://ftmscan.com/gastracker
            Self::Gnosis => 3_000_000_000,          // https://gnosisscan.io/gastracker
            Self::Blast => 200_000_000,             // https://blastscan.io/chart/gasprice
            Self::ZkSync => 20_000_000,             // https://era.zksync.network/chart/gasprice
            Self::Linea => 50_000_000,              // https://lineascan.build/gastracker
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
            Self::Optimism | Self::Base | Self::OpBNB | Self::World => true,
        }
    }

    pub fn weth_contract(&self) -> Option<&str> {
        match self {
            Self::Ethereum => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            Self::SmartChain => Some("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"),
            Self::Polygon => Some("0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),
            Self::Arbitrum => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
            Self::Optimism | Self::Base | Self::OpBNB | Self::World => Some("0x4200000000000000000000000000000000000006"),
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
        EVMChain::from_str(chain.as_ref()).ok()
    }

    pub fn to_chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chain, EVMChain};

    #[test]
    fn test_from_chain() {
        assert_eq!(EVMChain::from_chain(Chain::Ethereum), Some(EVMChain::Ethereum));
        assert_eq!(EVMChain::from_chain(Chain::Bitcoin), None);
    }
}
