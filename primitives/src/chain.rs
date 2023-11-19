use serde::{Serialize, Deserialize};
use typeshare::typeshare;
use strum::{IntoEnumIterator, EnumIter};

use crate::{AssetId, AssetType, ChainType};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
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
    Ripple,
    OpBNB,
    Fantom,
    Gnosis,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        return self.as_str() == other.as_str()
    }
}

impl Chain {
    pub fn from_str(chain: &str) -> Option<Self> {
        match chain {
            "bitcoin" => Some(Self::Bitcoin),
            "litecoin" => Some(Self::Litecoin),
            "binance" => Some(Self::Binance),
            "ethereum" => Some(Self::Ethereum),
            "smartchain" => Some(Self::SmartChain),
            "polygon" => Some(Self::Polygon),
            "solana" => Some(Self::Solana),
            "arbitrum" => Some(Self::Arbitrum),
            "optimism" => Some(Self::Optimism),
            "thorchain" => Some(Self::Thorchain),
            "cosmos" => Some(Self::Cosmos),
            "osmosis" => Some(Self::Osmosis),
            "ton" => Some(Self::Ton),
            "tron" => Some(Self::Tron),
            "doge" => Some(Self::Doge),
            "aptos" => Some(Self::Aptos),
            "base" => Some(Self::Base),
            "avalanchec"=> Some(Self::AvalancheC),
            "sui"=> Some(Self::Sui),
            "ripple"=> Some(Self::Ripple),
            "opbnb"=> Some(Self::OpBNB),
            "fantom"=> Some(Self::Fantom),
            "gnosis"=> Some(Self::Gnosis),
            _ => None, 
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Binance => "binance",
            Self::Bitcoin => "bitcoin",
            Self::Litecoin => "litecoin",
            Self::Ethereum => "ethereum",
            Self::SmartChain => "smartchain",
            Self::Polygon => "polygon",
            Self::Solana => "solana",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Thorchain => "thorchain",
            Self::Cosmos => "cosmos",
            Self::Osmosis => "osmosis",
            Self::Ton => "ton",
            Self::Tron => "tron",
            Self::Doge => "doge",
            Self::Aptos => "aptos",
            Self::Base => "base",
            Self::AvalancheC => "avalanchec",
            Self::Sui => "sui",
            Self::Ripple => "ripple",
            Self::OpBNB => "opbnb",
            Self::Fantom => "fantom",
            Self::Gnosis => "gnosis",
        }
    }

    pub fn as_denom(&self) -> &'static str {
        match self {
            Self::Binance => "BNB",
            Self::Thorchain => "rune",
            Self::Cosmos => "uatom",
            Self::Osmosis => "uosmo",
            Self::Sui => "0x2::sui::SUI",
            _ => {
                ""
            }
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
            Self::Bitcoin => todo!(),
            Self::Litecoin => todo!(),
            Self::Binance => todo!(),
            Self::Solana => todo!(),
            Self::Thorchain => todo!(),
            Self::Cosmos => todo!(),
            Self::Osmosis => todo!(),
            Self::Ton => todo!(),
            Self::Tron => todo!(),
            Self::Doge => todo!(),
            Self::Aptos => todo!(),
            Self::Sui => todo!(),
            Self::Ripple => todo!(),
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn is_utxo(&self) -> bool {
        match self {
            Self::Bitcoin | 
            Self::Litecoin | 
            Self::Doge => true,
            _ => {
                false
            }
        }
    }

    pub fn as_slip44(&self) -> i64 {
        match self {
            Self::Ethereum |
            Self::Fantom |
            Self::OpBNB |
            Self::Arbitrum |
            Self::Optimism |
            Self::Polygon |
            Self::Base |
            Self::Gnosis => 60,
            Self::Binance => 714,
            Self::Bitcoin => 0,
            Self::Litecoin => 2,
            Self::SmartChain => 9006,
            Self::Solana => 501,
            Self::Thorchain => 931,
            Self::Cosmos => 118,
            Self::Osmosis => 118,
            Self::Ton => 607,
            Self::Tron => 195,
            Self::Doge => 3,
            Self::Aptos => 637,
            Self::AvalancheC => 9005,
            Self::Sui => 784,
            Self::Ripple => 144,
        }
    }

    pub fn chain_type(&self) -> ChainType {
        match self {
            Self::Ethereum |
            Self::Fantom |
            Self::OpBNB |
            Self::Arbitrum |
            Self::Optimism |
            Self::Polygon |
            Self::Base |
            Self::SmartChain |
            Self::AvalancheC |
            Self::Gnosis => ChainType::Ethereum,
            Self::Binance => ChainType::Binance,
            Self::Bitcoin |
            Self::Doge |
            Self::Litecoin => ChainType::Bitcoin,
            Self::Solana => ChainType::Solana,
            Self::Thorchain |
            Self::Cosmos |
            Self::Osmosis => ChainType::Cosmos,
            Self::Ton => ChainType::Ton,
            Self::Tron => ChainType::Tron,
            Self::Aptos => ChainType::Aptos,
            Self::Sui => ChainType::Sui,
            Self::Ripple => ChainType::Ripple,
        }
    }

    pub fn default_asset_type(&self) -> Option<AssetType> {
        match self {
            Self::Ethereum |
            Self::Arbitrum |
            Self::Optimism  |
            Self::Polygon |
            Self::Base |
            Self::AvalancheC |
            Self::Gnosis |
            Self::Fantom => Some(AssetType::ERC20),
            Self::OpBNB |
            Self::SmartChain=> Some(AssetType::BEP20),
            Self::Binance => Some(AssetType::BEP2),
            Self::Solana => Some(AssetType::SPL),
            Self::Tron => Some(AssetType::TRC20),
            Self::Bitcoin |
            Self::Litecoin |
            Self::Thorchain |
            Self::Cosmos |
            Self::Osmosis |
            Self::Ton |
            Self::Doge |
            Self::Aptos |
            Self::Sui |
            Self::Ripple => None,
        }
    }

    pub fn all() -> Vec<Chain> {
        return Chain::iter().collect::<Vec<_>>();
    }
}