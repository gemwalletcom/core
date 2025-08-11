use primitives::Chain;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumIter)]
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

    /// Parse THORChain symbol to THORChainName
    /// Supports both long names (ETH, BSC, AVAX) and short names (e, s, a)
    pub fn from_symbol(symbol: &str) -> Option<THORChainName> {
        // Use strum's EnumIter to automatically iterate over all enum variants
        THORChainName::iter().find(|variant| variant.long_name() == symbol || variant.short_name() == symbol)
    }

    /// Parse THORChain memo to extract destination chain
    /// Memo format: =:ASSET:ADDRESS:LIMIT/INTERVAL/QUANTITY:AFFILIATE:BPS
    /// Example: "=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0:-_/ll:0/150"
    pub fn parse_dest_from_memo(memo: &str) -> Option<Chain> {
        // Split by ':' and get the asset part (index 1)
        let parts: Vec<&str> = memo.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let asset_part = parts[1];

        // Check if it's a token (contains '.')
        if let Some(dot_pos) = asset_part.find('.') {
            let chain_part = &asset_part[..dot_pos];
            THORChainName::from_symbol(chain_part).map(|thorchain_name| thorchain_name.chain())
        } else {
            // It's a native asset
            THORChainName::from_symbol(asset_part).map(|thorchain_name| thorchain_name.chain())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_symbol() {
        // Test long names (reusing existing long_name() method)
        assert_eq!(THORChainName::from_symbol("ETH"), Some(THORChainName::Ethereum));
        assert_eq!(THORChainName::from_symbol("BSC"), Some(THORChainName::SmartChain));
        assert_eq!(THORChainName::from_symbol("AVAX"), Some(THORChainName::AvalancheC));
        assert_eq!(THORChainName::from_symbol("BTC"), Some(THORChainName::Bitcoin));
        assert_eq!(THORChainName::from_symbol("LTC"), Some(THORChainName::Litecoin));
        assert_eq!(THORChainName::from_symbol("DOGE"), Some(THORChainName::Doge));
        assert_eq!(THORChainName::from_symbol("BCH"), Some(THORChainName::BitcoinCash));
        assert_eq!(THORChainName::from_symbol("GAIA"), Some(THORChainName::Cosmos));
        assert_eq!(THORChainName::from_symbol("THOR"), Some(THORChainName::Thorchain));
        assert_eq!(THORChainName::from_symbol("BASE"), Some(THORChainName::Base));
        assert_eq!(THORChainName::from_symbol("XRP"), Some(THORChainName::Xrp));

        // Test short names (reusing existing short_name() method)
        assert_eq!(THORChainName::from_symbol("e"), Some(THORChainName::Ethereum));
        assert_eq!(THORChainName::from_symbol("s"), Some(THORChainName::SmartChain));
        assert_eq!(THORChainName::from_symbol("a"), Some(THORChainName::AvalancheC));
        assert_eq!(THORChainName::from_symbol("b"), Some(THORChainName::Bitcoin));
        assert_eq!(THORChainName::from_symbol("l"), Some(THORChainName::Litecoin));
        assert_eq!(THORChainName::from_symbol("d"), Some(THORChainName::Doge));
        assert_eq!(THORChainName::from_symbol("c"), Some(THORChainName::BitcoinCash));
        assert_eq!(THORChainName::from_symbol("g"), Some(THORChainName::Cosmos));
        assert_eq!(THORChainName::from_symbol("r"), Some(THORChainName::Thorchain));
        assert_eq!(THORChainName::from_symbol("f"), Some(THORChainName::Base));
        assert_eq!(THORChainName::from_symbol("x"), Some(THORChainName::Xrp));

        // Test invalid symbols
        assert_eq!(THORChainName::from_symbol("INVALID"), None);
        assert_eq!(THORChainName::from_symbol("z"), None);
        assert_eq!(THORChainName::from_symbol(""), None);
    }

    #[test]
    fn test_parse_dest_from_memo() {
        // Test token swaps with long names
        assert_eq!(
            THORChainName::parse_dest_from_memo("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0:-_/ll:0/150"),
            Some(Chain::Ethereum)
        );

        assert_eq!(
            THORChainName::parse_dest_from_memo("=:BSC.USDT:0x123:0/1/0:affiliate:100"),
            Some(Chain::SmartChain)
        );

        assert_eq!(
            THORChainName::parse_dest_from_memo("=:AVAX.USDC:0x456:0/1/0:affiliate:50"),
            Some(Chain::AvalancheC)
        );

        // Test native asset swaps with short names
        assert_eq!(THORChainName::parse_dest_from_memo("=:e:0x789:0/1/0:affiliate:25"), Some(Chain::Ethereum));

        assert_eq!(THORChainName::parse_dest_from_memo("=:s:0xabc:0/1/0:affiliate:75"), Some(Chain::SmartChain));

        assert_eq!(THORChainName::parse_dest_from_memo("=:b:bc1qaddress:0/1/0:affiliate:100"), Some(Chain::Bitcoin));

        // Test other supported chains
        assert_eq!(
            THORChainName::parse_dest_from_memo("=:BTC.BTC:bc1qaddress:0/1/0:affiliate:150"),
            Some(Chain::Bitcoin)
        );

        assert_eq!(THORChainName::parse_dest_from_memo("=:DOGE.DOGE:D1234:0/1/0:affiliate:200"), Some(Chain::Doge));

        // Test invalid memos
        assert_eq!(THORChainName::parse_dest_from_memo("invalid:memo"), None);

        assert_eq!(THORChainName::parse_dest_from_memo("=:UNKNOWN.TOKEN:0x123:0/1/0:affiliate:50"), None);

        assert_eq!(THORChainName::parse_dest_from_memo("=:z:0x123:0/1/0:affiliate:50"), None);

        // Test edge cases
        assert_eq!(THORChainName::parse_dest_from_memo(""), None);

        assert_eq!(THORChainName::parse_dest_from_memo("="), None);
    }

    #[test]
    fn test_from_symbol_consistency_with_existing_methods() {
        // Ensure from_symbol works with all existing long/short names
        let all_variants = [
            THORChainName::Doge,
            THORChainName::Thorchain,
            THORChainName::Ethereum,
            THORChainName::Cosmos,
            THORChainName::Bitcoin,
            THORChainName::BitcoinCash,
            THORChainName::Litecoin,
            THORChainName::SmartChain,
            THORChainName::AvalancheC,
            THORChainName::Base,
            THORChainName::Xrp,
        ];

        for variant in all_variants.iter() {
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
}
