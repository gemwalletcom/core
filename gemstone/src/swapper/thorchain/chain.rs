use primitives::{Asset, Chain};

use super::asset::THORChainAsset;

#[derive(Clone)]
pub enum THORChainName {
    Doge,
    Thorchain,
    Ethereum,
    Cosmos,
    Bitcoin,
    Litecoin,
    SmartChain,
    AvalancheC,
}

// https://dev.thorchain.org/concepts/memo-length-reduction.html
impl THORChainName {
    pub fn short_name(&self) -> &str {
        match self {
            THORChainName::Doge => "d",       // DOGE.DOGE
            THORChainName::Thorchain => "r",  // THOR.RUNE
            THORChainName::Ethereum => "e",   // "ETH.ETH"
            THORChainName::Cosmos => "g",     // GAIA.ATOM
            THORChainName::Bitcoin => "b",    // BTC.BTC
            THORChainName::Litecoin => "l",   // LTC.LTC
            THORChainName::SmartChain => "s", // BSC.BNB
            THORChainName::AvalancheC => "a", // AVAX.AVAX
        }
    }

    pub fn long_name(&self) -> &str {
        match self {
            THORChainName::Doge => "DOGE",
            THORChainName::Thorchain => "THOR",
            THORChainName::Ethereum => "ETH",
            THORChainName::Cosmos => "GAIA",
            THORChainName::Bitcoin => "BTC",
            THORChainName::Litecoin => "LTC",
            THORChainName::SmartChain => "BSC",
            THORChainName::AvalancheC => "AVAX",
        }
    }

    pub fn chain(&self) -> Chain {
        match self {
            THORChainName::Doge => Chain::Doge,
            THORChainName::Thorchain => Chain::Thorchain,
            THORChainName::Ethereum => Chain::Ethereum,
            THORChainName::Cosmos => Chain::Cosmos,
            THORChainName::Bitcoin => Chain::Bitcoin,
            THORChainName::Litecoin => Chain::Litecoin,
            THORChainName::SmartChain => Chain::SmartChain,
            THORChainName::AvalancheC => Chain::AvalancheC,
        }
    }

    pub fn as_asset(&self) -> THORChainAsset {
        let asset = Asset::from_chain(self.chain());
        THORChainAsset {
            symbol: asset.symbol,
            chain: self.clone(),
            token_id: None,
            decimals: asset.decimals as u32,
        }
    }

    pub fn from_chain(chain: &Chain) -> Option<THORChainName> {
        match chain {
            Chain::Thorchain => Some(THORChainName::Thorchain),
            Chain::Doge => Some(THORChainName::Doge),
            Chain::Cosmos => Some(THORChainName::Cosmos),
            Chain::Bitcoin => Some(THORChainName::Bitcoin),
            Chain::Litecoin => Some(THORChainName::Litecoin),
            Chain::SmartChain => Some(THORChainName::SmartChain),
            Chain::Ethereum => Some(THORChainName::Ethereum),
            Chain::AvalancheC => Some(THORChainName::AvalancheC),
            _ => None,
        }
    }
}
