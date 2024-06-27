use crate::Chain;
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
    Blast,
    ZkSync,
    Linea,
    Mantle,
    Celo,
}

impl EVMChain {
    pub fn all() -> Vec<EVMChain> {
        EVMChain::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        match self {
            EVMChain::Ethereum => 1000000000, // https://etherscan.io/gastracker
            EVMChain::SmartChain => 1000000000, // https://bscscan.com/gastracker
            EVMChain::Polygon => 30000000000, // https://polygonscan.com/gastracker
            EVMChain::Arbitrum => 10000000, // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
            EVMChain::Optimism => 10000000, // https://optimistic.etherscan.io/chart/gasprice
            EVMChain::Base => 100000000,    // https://basescan.org/chart/gasprice
            EVMChain::AvalancheC => 25000000000, // https://snowscan.xyz/gastracker
            EVMChain::OpBNB => 1000000,     // https://opbnbscan.com/statistics
            EVMChain::Fantom => 3500000000, // https://ftmscan.com/gastracker
            EVMChain::Gnosis => 3000000000, // https://gnosisscan.io/gastracker
            EVMChain::Blast => 200000000,   // https://blastscan.io/chart/gasprice
            EVMChain::ZkSync => 20000000,   // https://era.zksync.network/chart/gasprice
            EVMChain::Linea => 50000000,    // https://lineascan.build/gastracker
            EVMChain::Mantle | EVMChain::Celo | EVMChain::Manta => 10000000,
        }
    }

    pub fn is_opstack(&self) -> bool {
        match self {
            EVMChain::Ethereum
            | EVMChain::SmartChain
            | EVMChain::Polygon
            | EVMChain::Arbitrum
            | EVMChain::AvalancheC
            | EVMChain::Fantom
            | EVMChain::Gnosis
            | EVMChain::Manta
            | EVMChain::Blast
            | EVMChain::ZkSync
            | EVMChain::Linea
            | EVMChain::Mantle
            | EVMChain::Celo => false,
            EVMChain::Optimism | EVMChain::Base | EVMChain::OpBNB => true,
        }
    }

    pub fn to_chain(&self) -> Chain {
        match self {
            EVMChain::Ethereum => Chain::Ethereum,
            EVMChain::SmartChain => Chain::SmartChain,
            EVMChain::Polygon => Chain::Polygon,
            EVMChain::Arbitrum => Chain::Arbitrum,
            EVMChain::Optimism => Chain::Optimism,
            EVMChain::Base => Chain::Base,
            EVMChain::AvalancheC => Chain::AvalancheC,
            EVMChain::OpBNB => Chain::OpBNB,
            EVMChain::Fantom => Chain::Fantom,
            EVMChain::Gnosis => Chain::Gnosis,
            EVMChain::Manta => Chain::Manta,
            EVMChain::Blast => Chain::Blast,
            EVMChain::ZkSync => Chain::ZkSync,
            EVMChain::Linea => Chain::Linea,
            EVMChain::Mantle => Chain::Mantle,
            EVMChain::Celo => Chain::Celo,
        }
    }
}
