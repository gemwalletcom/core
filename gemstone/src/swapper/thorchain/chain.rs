use primitives::Chain;

#[derive(Clone)]
pub enum THORChainName {
    Doge,
    Thorchain,
    Ethereum,
    Cosmos,
    Bitcoin,
    BitcoinCash,
    Litecoin,
    SmartChain,
    AvalancheC,
}

// https://dev.thorchain.org/concepts/memo-length-reduction.html
impl THORChainName {
    pub fn short_name(&self) -> &str {
        match self {
            THORChainName::Doge => "d",        // DOGE.DOGE
            THORChainName::Thorchain => "r",   // THOR.RUNE
            THORChainName::Ethereum => "e",    // "ETH.ETH"
            THORChainName::Cosmos => "g",      // GAIA.ATOM
            THORChainName::Bitcoin => "b",     // BTC.BTC
            THORChainName::BitcoinCash => "c", // BCH.BCH
            THORChainName::Litecoin => "l",    // LTC.LTC
            THORChainName::SmartChain => "s",  // BSC.BNB
            THORChainName::AvalancheC => "a",  // AVAX.AVAX
        }
    }

    pub fn long_name(&self) -> &str {
        match self {
            THORChainName::Doge => "DOGE",
            THORChainName::Thorchain => "THOR",
            THORChainName::Ethereum => "ETH",
            THORChainName::Cosmos => "GAIA",
            THORChainName::Bitcoin => "BTC",
            THORChainName::BitcoinCash => "BCH",
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
            THORChainName::BitcoinCash => Chain::BitcoinCash,
            THORChainName::Litecoin => Chain::Litecoin,
            THORChainName::SmartChain => Chain::SmartChain,
            THORChainName::AvalancheC => Chain::AvalancheC,
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
            Chain::BitcoinCash => Some(THORChainName::BitcoinCash),
            _ => None,
        }
    }

    pub fn is_evm_chain(&self) -> bool {
        match self {
            THORChainName::Ethereum | THORChainName::SmartChain | THORChainName::AvalancheC => true,
            THORChainName::Doge
            | THORChainName::Thorchain
            | THORChainName::Cosmos
            | THORChainName::Bitcoin
            | THORChainName::BitcoinCash
            | THORChainName::Litecoin => false,
        }
    }
}
