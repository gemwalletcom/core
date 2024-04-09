use std::fmt;

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{AssetId, AssetType, ChainType};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    Litecoin,
    Ethereum,
    Binance,
    SmartChain,
    Solana,
    Polygon,
    Thorchain,
    Cosmos,
    Osmosis,
    Arbitrum,
    Ton,
    Tron,
    Doge,
    Optimism,
    Aptos,
    Base,
    AvalancheC,
    Sui,
    Xrp,
    OpBNB,
    Fantom,
    Gnosis,
    Celestia,
    Injective,
    Sei,
    Manta,
    Blast,
    Noble,
    ZkSync,
    Linea,
    Mantle,
    Celo,
    Near,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self.as_ref())
    }
}

impl Chain {
    pub fn as_denom(&self) -> Option<&str> {
        match self {
            Self::Binance => Some("BNB"),
            Self::Thorchain => Some("rune"),
            Self::Cosmos => Some("uatom"),
            Self::Osmosis => Some("uosmo"),
            Self::Celestia => Some("utia"),
            Self::Injective => Some("inj"),
            Self::Sei => Some("usei"),
            Self::Noble => Some("uusdc"),
            Self::Sui => Some("0x2::sui::SUI"),
            Self::Aptos => Some("0x1::aptos_coin::AptosCoin"),
            _ => None,
        }
    }

    pub fn as_asset_id(&self) -> AssetId {
        AssetId::from_chain(*self)
    }

    pub fn network_id(&self) -> &str {
        match self {
            Self::Ethereum => "1",
            Self::SmartChain => "56",
            Self::Arbitrum => "42161",
            Self::AvalancheC => "43114",
            Self::Base => "8453",
            Self::Optimism => "10",
            Self::Polygon => "137",
            Self::OpBNB => "204",
            Self::Fantom => "250",
            Self::Gnosis => "100",
            Self::Manta => "169",
            Self::Blast => "81457",
            Self::Cosmos => "cosmoshub-4",
            Self::Osmosis => "osmosis-1",
            Self::Celestia => "celestia",
            Self::Noble => "noble-1",
            Self::Injective => "injective-1",
            Self::Sei => "pacific-1",
            Self::ZkSync => "324",
            Self::Linea => "59144",
            Self::Mantle => "5000",
            Self::Celo => "42220",
            Self::Near => "1",
            Self::Bitcoin
            | Self::Litecoin
            | Self::Binance
            | Self::Solana
            | Self::Thorchain
            | Self::Ton
            | Self::Tron
            | Self::Doge
            | Self::Aptos
            | Self::Sui
            | Self::Xrp => unimplemented!(),
        }
    }

    pub fn is_utxo(&self) -> bool {
        matches!(self, Self::Bitcoin | Self::Litecoin | Self::Doge)
    }

    pub fn as_slip44(&self) -> i64 {
        match self {
            Self::Ethereum
            | Self::Fantom
            | Self::OpBNB
            | Self::Arbitrum
            | Self::Optimism
            | Self::Polygon
            | Self::Base
            | Self::Gnosis
            | Self::Injective
            | Self::Manta
            | Self::Blast
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo => 60,
            Self::Binance => 714,
            Self::Bitcoin => 0,
            Self::Litecoin => 2,
            Self::SmartChain => 9006,
            Self::Solana => 501,
            Self::Thorchain => 931,
            Self::Cosmos | Self::Osmosis | Self::Celestia | Self::Noble | Self::Sei => 118,
            Self::Ton => 607,
            Self::Tron => 195,
            Self::Doge => 3,
            Self::Aptos => 637,
            Self::AvalancheC => 9005,
            Self::Sui => 784,
            Self::Xrp => 144,
            Self::Near => 397,
        }
    }

    pub fn chain_type(&self) -> ChainType {
        match self {
            Self::Ethereum
            | Self::Fantom
            | Self::OpBNB
            | Self::Arbitrum
            | Self::Optimism
            | Self::Polygon
            | Self::Base
            | Self::SmartChain
            | Self::AvalancheC
            | Self::Gnosis
            | Self::Manta
            | Self::Blast
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo => ChainType::Ethereum,
            Self::Binance => ChainType::Binance,
            Self::Bitcoin | Self::Doge | Self::Litecoin => ChainType::Bitcoin,
            Self::Solana => ChainType::Solana,
            Self::Thorchain
            | Self::Cosmos
            | Self::Osmosis
            | Self::Celestia
            | Self::Injective
            | Self::Noble
            | Self::Sei => ChainType::Cosmos,
            Self::Ton => ChainType::Ton,
            Self::Tron => ChainType::Tron,
            Self::Aptos => ChainType::Aptos,
            Self::Sui => ChainType::Sui,
            Self::Xrp => ChainType::Xrp,
            Self::Near => ChainType::Near,
        }
    }

    pub fn default_asset_type(&self) -> Option<AssetType> {
        match self {
            Self::Ethereum
            | Self::Arbitrum
            | Self::Optimism
            | Self::Polygon
            | Self::Base
            | Self::AvalancheC
            | Self::Gnosis
            | Self::Fantom
            | Self::Manta
            | Self::Blast
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo => Some(AssetType::ERC20),
            Self::OpBNB | Self::SmartChain => Some(AssetType::BEP20),
            Self::Binance => Some(AssetType::BEP2),
            Self::Solana => Some(AssetType::SPL),
            Self::Tron => Some(AssetType::TRC20),
            Self::Bitcoin
            | Self::Litecoin
            | Self::Thorchain
            | Self::Cosmos
            | Self::Osmosis
            | Self::Ton
            | Self::Doge
            | Self::Aptos
            | Self::Sui
            | Self::Xrp
            | Self::Celestia
            | Self::Injective
            | Self::Noble
            | Self::Sei
            | Self::Near => None,
        }
    }

    // miliseconds
    pub fn block_time(&self) -> i64 {
        match self {
            Self::Ethereum => 12_000,
            Self::Manta => 12_000,
            Self::Fantom => 1_000,
            Self::OpBNB => 1_000,
            Self::Arbitrum => 1_000,
            Self::Optimism | Self::Blast => 2_000,
            Self::Polygon => 3_000,
            Self::Base => 2_000,
            Self::Gnosis => 5_000,
            Self::Binance => 500,
            Self::Bitcoin => 600_000,
            Self::Litecoin => 120_000,
            Self::SmartChain => 3_000,
            Self::Solana => 500,
            Self::Thorchain => 10_000,
            Self::Cosmos => 6_000,
            Self::Osmosis => 6_000,
            Self::Celestia => 6_000,
            Self::Injective => 6_000,
            Self::Ton => 5_000,
            Self::Tron => 3_000,
            Self::Doge => 60_000,
            Self::Aptos => 500,
            Self::AvalancheC => 2_000,
            Self::Sui => 500,
            Self::Xrp => 4_000,
            Self::Sei => 1_000,
            Self::Noble => 6_000,
            Self::ZkSync => 1_000,
            Self::Linea => 1_000,
            Self::Mantle => 1_000,
            Self::Celo => 1_000,
            Self::Near => 1_000,
        }
    }

    pub fn rank(&self) -> i32 {
        match self {
            Self::Bitcoin => 100,
            Self::Ethereum => 80,
            Self::Solana | Self::SmartChain => 70,
            Self::Osmosis | Self::Ton | Self::Tron => 50,
            Self::Cosmos
            | Self::Injective
            | Self::Aptos
            | Self::Sui
            | Self::Xrp
            | Self::Celestia => 40,
            Self::Manta
            | Self::Fantom
            | Self::OpBNB
            | Self::Arbitrum
            | Self::Blast
            | Self::Optimism
            | Self::Polygon
            | Self::Base
            | Self::Gnosis
            | Self::Thorchain
            | Self::Doge
            | Self::AvalancheC
            | Self::Sei
            | Self::Noble
            | Self::Litecoin
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo
            | Self::Near => 30,
            Self::Binance => 0,
        }
    }

    pub fn all() -> Vec<Chain> {
        Chain::iter().collect::<Vec<_>>()
    }
}
