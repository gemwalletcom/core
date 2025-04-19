use crate::Chain;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, PartialEq)]
pub enum ChainStack {
    Native,
    Optimism,
    ZkSync,
}

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
    Sonic,
    Abstract,
    Berachain,
    Ink,
    Unichain,
    Hyperliquid,
    Monad,
}

impl EVMChain {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        match self {
            Self::Ethereum => 1_000_000_000,                                                      // https://etherscan.io/gastracker
            Self::SmartChain => 1_000_000_000,                                                    // https://bscscan.com/gastracker
            Self::Polygon => 30_000_000_000,                                                      // https://polygonscan.com/gastracker
            Self::Arbitrum => 10_000_000, // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
            Self::Optimism => 10_000_000, // https://optimistic.etherscan.io/chart/gasprice
            Self::Base => 100_000_000,    // https://basescan.org/chart/gasprice
            Self::AvalancheC => 25_000_000_000, // https://snowscan.xyz/gastracker
            Self::OpBNB | Self::World | Self::Abstract | Self::Ink | Self::Unichain => 1_000_000, // https://opbnbscan.com/statistics
            Self::Fantom => 3_500_000_000, // https://ftmscan.com/gastracker
            Self::Gnosis => 3_000_000_000, // https://gnosisscan.io/gastracker
            Self::Blast => 200_000_000,   // https://blastscan.io/chart/gasprice
            Self::ZkSync => 20_000_000,   // https://era.zksync.network/chart/gasprice
            Self::Linea => 50_000_000,    // https://lineascan.build/gastracker
            Self::Mantle | Self::Celo | Self::Manta => 10_000_000,
            Self::Sonic => 10_000_000,
            Self::Berachain => 1_000_000_000,   // 1 Gwei
            Self::Hyperliquid => 1_000_000_000, // 1 Gwei
            Self::Monad => 1_000_000_000,       // 1 Gwei
        }
    }

    pub fn chain_stack(&self) -> ChainStack {
        match self {
            Self::Optimism | Self::Base | Self::OpBNB | Self::World | Self::Ink | Self::Unichain | Self::Celo => ChainStack::Optimism,
            Self::ZkSync | Self::Abstract => ChainStack::ZkSync,
            Self::Ethereum
            | Self::SmartChain
            | Self::Polygon
            | Self::Arbitrum
            | Self::AvalancheC
            | Self::Fantom
            | Self::Gnosis
            | Self::Manta
            | Self::Blast
            | Self::Linea
            | Self::Mantle
            | Self::Sonic
            | Self::Berachain
            | Self::Hyperliquid
            | Self::Monad => ChainStack::Native,
        }
    }

    pub fn is_ethereum_layer2(&self) -> bool {
        matches!(
            self,
            Self::Abstract
                | Self::Optimism
                | Self::Base
                | Self::World
                | Self::Ink
                | Self::Unichain
                | Self::ZkSync
                | Self::Arbitrum
                | Self::Blast
                | Self::Linea
                | Self::Celo
                | Self::Mantle
        )
    }

    // https://docs.optimism.io/stack/getting-started
    pub fn is_opstack(&self) -> bool {
        self.chain_stack() == ChainStack::Optimism
    }

    // https://docs.zksync.io/zk-stack/running/quickstart
    pub fn is_zkstack(&self) -> bool {
        self.chain_stack() == ChainStack::ZkSync
    }

    pub fn weth_contract(&self) -> Option<&str> {
        match self {
            Self::Ethereum => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            Self::SmartChain => Some("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"), // WBNB
            Self::Polygon => Some("0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),    // WMATIC
            Self::Arbitrum => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
            Self::Optimism | Self::Base | Self::OpBNB | Self::World | Self::Ink | Self::Unichain => Some("0x4200000000000000000000000000000000000006"),
            Self::AvalancheC => Some("0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7"), // WAVAX
            Self::Fantom => Some("0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83"),     // WFTM
            Self::Gnosis => Some("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d"),     // Wrapped XDAI (WXDAI)
            Self::ZkSync => Some("0x5AEa5775959fBC2557Cc8789bC1bf90A239D9a91"),
            Self::Blast => Some("0x4300000000000000000000000000000000000004"),
            Self::Celo => Some("0x471EcE3750Da237f93B8E339c536989b8978a438"),
            Self::Sonic => Some("0x039e2fB66102314Ce7b64Ce5Ce3E5183bc94aD38"), // Wrapped Sonic (wS)
            Self::Abstract => Some("0x3439153EB7AF838Ad19d56E1571FBD09333C2809"),
            Self::Berachain => Some("0x6969696969696969696969696969696969696969"), // WBERA
            Self::Hyperliquid => Some("0x5555555555555555555555555555555555555555"), // WHYPE
            Self::Linea => Some("0xe5D7C2a44FfDDf6b295A15c148167daaAf5Cf34f"),
            Self::Mantle => Some("0x78c1b0C915c4FAA5FffA6CAbf0219DA63d7f4cb8"), // Wrapped Mantle (WMNT)
            Self::Manta => Some("0x0dc808adce2099a9f62aa87d9670745aba741746"),
            Self::Monad => None, //TODO: Monad
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
