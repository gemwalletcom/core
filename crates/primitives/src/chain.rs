use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use strum_macros::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{AssetId, AssetType, ChainType, StakeChain};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    BitcoinCash,
    Litecoin,
    Ethereum,
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
    World,
    Stellar,
    Sonic,
    Algorand,
    Polkadot,
    Cardano,
    Abstract,
    Berachain,
    Ink,
    Unichain,
    HyperEvm,
    Monad,
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
            Self::World => "480",
            Self::Cosmos => "cosmoshub-4",
            Self::Osmosis => "osmosis-1",
            Self::Celestia => "celestia",
            Self::Noble => "noble-1",
            Self::Injective => "injective-1",
            Self::Sei => "pacific-1",
            Self::Thorchain => "thorchain-1",
            Self::ZkSync => "324",
            Self::Linea => "59144",
            Self::Mantle => "5000",
            Self::Celo => "42220",
            Self::Near => "mainnet",
            Self::Bitcoin | Self::BitcoinCash => "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            Self::Litecoin => "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2",
            Self::Doge => "1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691",
            Self::Solana => "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d",
            Self::Ton => "F6OpKZKqvqeFp6CQmFomXNMfMj2EnaUSOXN+Mh+wVWk=",
            Self::Sui => "35834a8a", // https://docs.sui.io/sui-api-ref#sui_getchainidentifier
            Self::Aptos => "1",
            Self::Tron | Self::Xrp => "",
            Self::Stellar => "Public Global Stellar Network ; September 2015",
            Self::Sonic => "146",
            Self::Algorand => "mainnet-v1.0",
            Self::Polkadot => "Polkadot",
            Self::Cardano => "764824073", // magic number from genesis configuration
            Self::Abstract => "2741",
            Self::Berachain => "80094",
            Self::Ink => "57073",
            Self::Unichain => "130",
            Self::HyperEvm => "999",
            Self::Monad => "10143", //TODO: Monad 143
        }
    }

    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        Self::iter().find(|&x| x.network_id() == chain_id.to_string())
    }

    pub fn is_utxo(&self) -> bool {
        matches!(self, Self::Bitcoin | Self::Litecoin | Self::Doge | Self::Cardano)
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
            | Self::Celo
            | Self::World
            | Self::Sonic
            | Self::Abstract
            | Self::Berachain
            | Self::Ink
            | Self::Unichain
            | Self::HyperEvm
            | Self::Monad => 60,
            Self::Bitcoin => 0,
            Self::BitcoinCash => 145,
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
            Self::Stellar => 148,
            Self::Algorand => 283,
            Self::Polkadot => 354,
            Self::Cardano => 1815,
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
            | Self::Celo
            | Self::World
            | Self::Sonic
            | Self::Abstract
            | Self::Berachain
            | Self::Ink
            | Self::Unichain
            | Self::HyperEvm
            | Self::Monad => ChainType::Ethereum,
            Self::Bitcoin | Self::BitcoinCash | Self::Doge | Self::Litecoin => ChainType::Bitcoin,
            Self::Solana => ChainType::Solana,
            Self::Thorchain | Self::Cosmos | Self::Osmosis | Self::Celestia | Self::Injective | Self::Noble | Self::Sei => ChainType::Cosmos,
            Self::Ton => ChainType::Ton,
            Self::Tron => ChainType::Tron,
            Self::Aptos => ChainType::Aptos,
            Self::Sui => ChainType::Sui,
            Self::Xrp => ChainType::Xrp,
            Self::Near => ChainType::Near,
            Self::Stellar => ChainType::Stellar,
            Self::Algorand => ChainType::Algorand,
            Self::Polkadot => ChainType::Polkadot,
            Self::Cardano => ChainType::Cardano,
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
            | Self::Celo
            | Self::World
            | Self::Sonic
            | Self::Abstract
            | Self::Berachain
            | Self::Ink
            | Self::Unichain
            | Self::HyperEvm
            | Self::Monad => Some(AssetType::ERC20),
            Self::OpBNB | Self::SmartChain => Some(AssetType::BEP20),
            Self::Solana => Some(AssetType::SPL),
            Self::Tron => Some(AssetType::TRC20),
            Self::Ton => Some(AssetType::JETTON),
            Self::Sui | Self::Aptos => Some(AssetType::TOKEN),
            Self::Algorand => Some(AssetType::ASA),
            Self::Bitcoin
            | Self::BitcoinCash
            | Self::Litecoin
            | Self::Thorchain
            | Self::Cosmos
            | Self::Osmosis
            | Self::Doge
            | Self::Xrp
            | Self::Celestia
            | Self::Injective
            | Self::Noble
            | Self::Sei
            | Self::Near
            | Self::Stellar
            | Self::Polkadot
            | Self::Cardano => None,
        }
    }

    pub fn account_activation_fee(&self) -> Option<i32> {
        match self {
            Self::Xrp => Some(1_000_000), // https://xrpl.org/docs/concepts/accounts/reserves#base-reserve-and-owner-reserve
            Self::Stellar => Some(10_000_000),
            Self::Algorand => Some(100_000),
            _ => None,
        }
    }

    pub fn token_activation_fee(&self) -> Option<i32> {
        match self {
            Self::Xrp => Some(200_000),    // https://xrpl.org/docs/concepts/accounts/reserves#base-reserve-and-owner-reserve
            Self::Solana => Some(2039280), // 2039280 (165 bytes) https://solana.com/docs/core/accounts
            _ => None,
        }
    }

    pub fn minimum_account_balance(&self) -> Option<u64> {
        match self {
            Self::Solana => Some(890_880),
            Self::Polkadot => Some(10_000_000_000),
            _ => None,
        }
    }

    pub fn is_swap_supported(&self) -> bool {
        match self {
            Self::Ethereum
            | Self::Bitcoin
            | Self::BitcoinCash
            | Self::Litecoin
            | Self::SmartChain
            | Self::Cosmos
            | Self::Fantom
            | Self::OpBNB
            | Self::Arbitrum
            | Self::Optimism
            | Self::Polygon
            | Self::Base
            | Self::Gnosis
            | Self::Manta
            | Self::Blast
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo
            | Self::World
            | Self::Thorchain
            | Self::Solana
            | Self::AvalancheC
            | Self::Doge
            | Self::Aptos
            | Self::Sonic
            | Self::Abstract
            | Self::Unichain
            | Self::Ink
            | Self::HyperEvm
            | Self::Sui
            | Self::Monad
            | Self::Ton => true,
            Self::Osmosis
            | Self::Celestia
            | Self::Injective
            | Self::Tron
            | Self::Xrp
            | Self::Sei
            | Self::Noble
            | Self::Near
            | Self::Stellar
            | Self::Algorand
            | Self::Polkadot
            | Self::Cardano
            | Self::Berachain => false,
        }
    }

    pub fn is_stake_supported(&self) -> bool {
        StakeChain::from_str(self.as_ref()).is_ok()
    }

    pub fn is_nft_supported(&self) -> bool {
        matches!(self, Self::Ethereum | Self::Solana)
    }

    // milliseconds
    pub fn block_time(&self) -> u32 {
        match self {
            Self::Solana | Self::Aptos | Self::Sui | Self::Monad | Self::Sonic => 500,
            // 1,000 ms
            Self::Arbitrum
            | Self::Celo
            | Self::Fantom
            | Self::Ink
            | Self::Linea
            | Self::Mantle
            | Self::Near
            | Self::OpBNB
            | Self::Sei
            | Self::Unichain
            | Self::ZkSync
            | Self::Abstract
            | Self::SmartChain => 1_000,
            Self::AvalancheC
            | Self::Base
            | Self::Blast
            | Self::HyperEvm
            | Self::Manta
            | Self::Optimism
            | Self::World
            | Self::Berachain
            | Self::Thorchain => 2_000,
            Self::Polygon | Self::Tron => 3_000,
            Self::Algorand | Self::Xrp => 4_000,
            Self::Gnosis | Self::Polkadot | Self::Ton => 5_000,
            Self::Celestia | Self::Cosmos | Self::Injective | Self::Noble | Self::Osmosis | Self::Stellar => 6_000,
            Self::Ethereum => 12_000,
            Self::Cardano => 20_000,
            Self::Doge => 60_000,
            Self::Litecoin => 120_000,
            Self::Bitcoin | Self::BitcoinCash => 600_000,
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
            | Self::Celestia
            | Self::BitcoinCash
            | Self::Polkadot
            | Self::HyperEvm
            | Self::Monad => 40,
            Self::Abstract | Self::Berachain | Self::Ink | Self::Unichain => 35,
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
            | Self::Litecoin
            | Self::ZkSync
            | Self::Linea
            | Self::Mantle
            | Self::Celo
            | Self::Near
            | Self::World
            | Self::Stellar
            | Self::Sonic
            | Self::Algorand
            | Self::Cardano => 30,
            Self::Noble => 20,
        }
    }

    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}
