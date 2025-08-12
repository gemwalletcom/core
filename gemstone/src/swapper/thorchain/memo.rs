use super::chain::THORChainName;
use primitives::Chain;

#[derive(Debug, Clone, PartialEq)]
pub struct ThorchainMemo {
    pub tx_type: String,
    pub asset: String,
    pub address: String,
}

impl ThorchainMemo {
    /// Parse a THORChain memo string into structured data
    pub fn parse(memo: &str) -> Option<Self> {
        if memo.is_empty() {
            return None;
        }

        let parts: Vec<&str> = memo.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        Some(ThorchainMemo {
            tx_type: parts[0].to_string(),
            asset: parts[1].to_string(),
            address: parts[2].to_string(),
        })
    }

    pub fn destination_chain(&self) -> Option<Chain> {
        // Check if it's a token (contains '.')
        if let Some(dot_pos) = self.asset.find('.') {
            let chain_part = &self.asset[..dot_pos];
            THORChainName::from_symbol(chain_part).map(|thorchain_name| thorchain_name.chain())
        } else {
            THORChainName::from_symbol(&self.asset).map(|thorchain_name| thorchain_name.chain())
        }
    }

    #[allow(dead_code)]
    pub fn token_symbol(&self) -> Option<String> {
        self.asset.find('.').map(|dot_pos| self.asset[dot_pos + 1..].to_string())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_memo() {
        let memo = "=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0:-_/ll:0/150";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.tx_type, "=");
        assert_eq!(parsed.asset, "ETH.USDT");
        assert_eq!(parsed.address, "0x858734a6353C9921a78fB3c937c8E20Ba6f36902");
        assert_eq!(parsed.destination_chain(), Some(Chain::Ethereum));
        assert_eq!(parsed.token_symbol(), Some("USDT".to_string()));
    }

    #[test]
    fn test_parse_simple_swap() {
        let memo = "=:ETH:0x858734a6353C9921a78fB3c937c8E20Ba6f36902";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.tx_type, "=");
        assert_eq!(parsed.asset, "ETH");
        assert_eq!(parsed.address, "0x858734a6353C9921a78fB3c937c8E20Ba6f36902");
        assert_eq!(parsed.destination_chain(), Some(Chain::Ethereum));
        assert_eq!(parsed.token_symbol(), None);
    }

    #[test]
    fn test_parse_short_names() {
        let memo = "=:e:0x858734a6353C9921a78fB3c937c8E20Ba6f36902";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.asset, "e");
        assert_eq!(parsed.destination_chain(), Some(Chain::Ethereum));
    }

    #[test]
    fn test_parse_bitcoin_memo() {
        let memo = "=:BTC:bc1qaddress:0/1/0:affiliate:150";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.destination_chain(), Some(Chain::Bitcoin));
        assert_eq!(parsed.token_symbol(), None);
    }

    #[test]
    fn test_parse_bsc_token() {
        let memo = "=:BSC.USDT:0x123:0/1/0:affiliate:100";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        assert_eq!(parsed.destination_chain(), Some(Chain::SmartChain));
        assert_eq!(parsed.token_symbol(), Some("USDT".to_string()));
    }


    #[test]
    fn test_parse_invalid_memos() {
        assert!(ThorchainMemo::parse("").is_none());
        assert!(ThorchainMemo::parse("invalid").is_none());
        assert!(ThorchainMemo::parse("=:").is_none());
        assert!(ThorchainMemo::parse("=:ETH").is_none());
    }

    #[test]
    fn test_parse_unknown_chain() {
        let memo = "=:UNKNOWN.TOKEN:0x123";
        let parsed = ThorchainMemo::parse(memo).unwrap();
        assert_eq!(parsed.destination_chain(), None);
    }
}
