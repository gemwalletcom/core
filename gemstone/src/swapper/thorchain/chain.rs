use primitives::Chain;

#[derive(Clone, PartialEq, Eq)]
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
    Base,
    Xrp,
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
            THORChainName::Base => "f",        // BASE.ETH
            THORChainName::Xrp => "x",         // XRP.XRP
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
            THORChainName::Base => "BASE",
            THORChainName::Xrp => "XRP",
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
            THORChainName::Base => Chain::Base,
            THORChainName::Xrp => Chain::Xrp,
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
            Chain::Base => Some(THORChainName::Base),
            Chain::Xrp => Some(THORChainName::Xrp),
            _ => None,
        }
    }

    pub fn is_evm_chain(&self) -> bool {
        match self {
            THORChainName::Ethereum | THORChainName::SmartChain | THORChainName::AvalancheC | THORChainName::Base => true,
            THORChainName::Doge
            | THORChainName::Thorchain
            | THORChainName::Cosmos
            | THORChainName::Bitcoin
            | THORChainName::BitcoinCash
            | THORChainName::Litecoin
            | THORChainName::Xrp => false,
        }
    }
}
