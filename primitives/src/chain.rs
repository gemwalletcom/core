use serde::{Serialize, Deserialize};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Bitcoin,
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
    pub fn new(chain: &str) -> Option<Self> {
        match chain {
            "bitcoin" => Some(Self::Bitcoin),
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

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}