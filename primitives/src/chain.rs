use serde::{Serialize, Deserialize};
use typeshare::typeshare;
use strum::{IntoEnumIterator, EnumIter};

use crate::{AssetId, AssetType};

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
            Self::Binance => 714,
            Self::Bitcoin => 0,
            Self::Litecoin => 2,
            Self::Ethereum => 60,
            Self::SmartChain => 9006,
            Self::Polygon => 60,
            Self::Solana => 501,
            Self::Arbitrum => 60,
            Self::Optimism => 60,
            Self::Thorchain => 931,
            Self::Cosmos => 118,
            Self::Osmosis => 118,
            Self::Ton => 607,
            Self::Tron => 195,
            Self::Doge => 3,
            Self::Aptos => 637,
            Self::Base => 60,
            Self::AvalancheC => 9005,
            Self::Sui => 784,
            Self::Ripple => 144,
            Self::OpBNB => 60,
        }
    }

    pub fn default_asset_type(&self) -> Option<AssetType> {
        match self {
            Self::Binance => Some(AssetType::BEP2),
            Self::Bitcoin => None,
            Self::Litecoin => None,
            Self::Ethereum => Some(AssetType::ERC20),
            Self::SmartChain => Some(AssetType::BEP20),
            Self::Polygon => Some(AssetType::ERC20),
            Self::Solana => Some(AssetType::SPL),
            Self::Arbitrum => Some(AssetType::ERC20),
            Self::Optimism => Some(AssetType::ERC20),
            Self::Thorchain => None,
            Self::Cosmos => None,
            Self::Osmosis => None,
            Self::Ton => None,
            Self::Tron => Some(AssetType::TRC20),
            Self::Doge => None,
            Self::Aptos => None,
            Self::Base => Some(AssetType::ERC20),
            Self::AvalancheC => Some(AssetType::ERC20),
            Self::Sui => None,
            Self::Ripple => None,
            Self::OpBNB => Some(AssetType::BEP20),
        }
    }

    pub fn all() -> Vec<Chain> {
        return Chain::iter().collect::<Vec<_>>();
    }
}