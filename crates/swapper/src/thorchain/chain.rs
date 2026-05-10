use gem_evm::address::ethereum_address_checksum;
use primitives::Chain;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
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
    Tron,
    Solana,
    Zcash,
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
            THORChainName::Tron => "tr",       // TRON.TRX
            THORChainName::Solana => "o",      // SOL.SOL
            THORChainName::Zcash => "z",       // ZEC.ZEC
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
            THORChainName::Tron => "TRON",
            THORChainName::Solana => "SOL",
            THORChainName::Zcash => "ZEC",
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
            THORChainName::Tron => Chain::Tron,
            THORChainName::Solana => Chain::Solana,
            THORChainName::Zcash => Chain::Zcash,
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
            Chain::Tron => Some(THORChainName::Tron),
            Chain::Solana => Some(THORChainName::Solana),
            Chain::Zcash => Some(THORChainName::Zcash),
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
            | THORChainName::Xrp
            | THORChainName::Tron
            | THORChainName::Solana
            | THORChainName::Zcash => false,
        }
    }

    pub fn from_symbol(symbol: &str) -> Option<THORChainName> {
        THORChainName::iter().find(|variant| variant.long_name() == symbol || variant.short_name() == symbol)
    }

    pub fn checksum_address(&self, address: &str) -> String {
        if self.is_evm_chain() {
            let address = address.strip_prefix("0X").unwrap_or(address);
            ethereum_address_checksum(address).unwrap_or(address.to_string())
        } else {
            address.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_symbol() {
        // Ensure from_symbol works with all existing long/short names
        for variant in THORChainName::iter() {
            // Test that long names can be parsed back
            assert_eq!(
                THORChainName::from_symbol(variant.long_name()),
                Some(variant.clone()),
                "Failed to parse long name: {}",
                variant.long_name()
            );

            // Test that short names can be parsed back
            assert_eq!(
                THORChainName::from_symbol(variant.short_name()),
                Some(variant.clone()),
                "Failed to parse short name: {}",
                variant.short_name()
            );
        }
    }

    #[test]
    fn test_zcash_mapping() {
        assert_eq!(THORChainName::Zcash.short_name(), "z");
        assert_eq!(THORChainName::Zcash.long_name(), "ZEC");
        assert_eq!(THORChainName::Zcash.chain(), Chain::Zcash);
        assert_eq!(THORChainName::from_chain(&Chain::Zcash), Some(THORChainName::Zcash));
        assert_eq!(THORChainName::from_symbol("ZEC"), Some(THORChainName::Zcash));
        assert_eq!(THORChainName::from_symbol("z"), Some(THORChainName::Zcash));
    }

    #[test]
    fn test_checksum_address_preserves_non_evm_case() {
        let zcash = "t1Ku2KLyndDPsR32jwnrTMd3yvi9tfFP8ML";
        assert_eq!(THORChainName::Zcash.checksum_address(zcash), zcash);
    }
}
