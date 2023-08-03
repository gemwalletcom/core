use serde::{Serializer, Serialize, Deserialize, Deserializer};
use typeshare::typeshare;

#[derive(Copy, Clone, Debug)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
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
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        return self.as_str() == other.as_str()
    }
}

impl Serialize for Chain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let result = Self::new(s.as_str());
        
        match result {
            Some(result) => Ok(result),
            _ => Err(serde::de::Error::custom(format!("Invalid chain: {}", s))),
        }
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
        }
    }
}