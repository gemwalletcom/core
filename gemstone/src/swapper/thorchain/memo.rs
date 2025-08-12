use super::chain::THORChainName;
use primitives::Chain;

#[derive(Debug, Clone, PartialEq)]
pub struct ThorchainMemo {
    pub tx_type: String,
    pub asset: String,
    pub address: String,
    pub streaming_swap: Option<StreamingSwapParams>,
    pub affiliate: Option<AffiliateParams>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamingSwapParams {
    pub limit: Option<u64>,
    pub interval: Option<u64>,
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AffiliateParams {
    pub address: String,
    pub fee_bps: u64,
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

        let tx_type = parts[0].to_string();
        let asset = parts[1].to_string();
        let address = parts[2].to_string();

        // Parse optional streaming swap parameters
        let streaming_swap = if parts.len() > 3 && !parts[3].is_empty() {
            Self::parse_streaming_swap(parts[3])
        } else {
            None
        };

        // Parse optional affiliate parameters
        let affiliate = if parts.len() > 7 {
            // Format: =:ASSET:ADDRESS::::affiliate:BPS
            Self::parse_affiliate(parts[6], parts[7])
        } else if parts.len() > 6 {
            // Format: =:ASSET:ADDRESS:::affiliate:BPS
            Self::parse_affiliate(parts[5], parts[6])
        } else if parts.len() > 5 {
            // Format: =:ASSET:ADDRESS:STREAMING:affiliate:BPS
            Self::parse_affiliate(parts[4], parts[5])
        } else {
            None
        };

        Some(ThorchainMemo {
            tx_type,
            asset,
            address,
            streaming_swap,
            affiliate,
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

    /// Get the total estimated fee in basis points
    #[allow(dead_code)]
    pub fn total_fee_bps(&self) -> u64 {
        self.affiliate.as_ref().map_or(0, |a| a.fee_bps)
    }

    fn parse_streaming_swap(param: &str) -> Option<StreamingSwapParams> {
        // Handle special case of "///" (empty streaming params)
        if param == "///" {
            return Some(StreamingSwapParams {
                limit: None,
                interval: None,
                quantity: None,
            });
        }

        let parts: Vec<&str> = param.split('/').collect();
        if parts.len() != 3 {
            return None;
        }

        let limit = if parts[0].is_empty() {
            None
        } else {
            // Handle scientific notation like "1635978e6"
            if parts[0].contains('e') || parts[0].contains('E') {
                parts[0].parse::<f64>().ok().map(|f| f as u64)
            } else {
                parts[0].parse().ok()
            }
        };
        let interval = if parts[1].is_empty() { None } else { parts[1].parse().ok() };
        let quantity = if parts[2].is_empty() { None } else { parts[2].parse().ok() };

        Some(StreamingSwapParams { limit, interval, quantity })
    }

    fn parse_affiliate(address: &str, fee_bps: &str) -> Option<AffiliateParams> {
        if address.is_empty() && fee_bps.is_empty() {
            return None;
        }

        // Handle the case where fee_bps might be in format "0/150"
        let (affiliate_address, fee) = if fee_bps.contains('/') {
            let parts: Vec<&str> = fee_bps.split('/').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].parse().ok()?)
            } else {
                (address.to_string(), fee_bps.parse().ok()?)
            }
        } else {
            (address.to_string(), fee_bps.parse().ok()?)
        };

        Some(AffiliateParams {
            address: affiliate_address,
            fee_bps: fee,
        })
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
        assert!(parsed.streaming_swap.is_some());

        assert_eq!(parsed.total_fee_bps(), 150);

        let streaming = parsed.streaming_swap.as_ref().unwrap();
        assert_eq!(streaming.limit, Some(1635978000000));
        assert_eq!(streaming.interval, Some(1));
        assert_eq!(streaming.quantity, Some(0));

        let affiliate = parsed.affiliate.as_ref().unwrap();
        assert_eq!(affiliate.address, "0");
        assert_eq!(affiliate.fee_bps, 150);
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
        assert!(parsed.streaming_swap.is_none());
        assert_eq!(parsed.total_fee_bps(), 0);
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
    fn test_parse_with_streaming_swap() {
        let memo = "=:AVAX.USDC:0x456:1000/5/10";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        let streaming = parsed.streaming_swap.as_ref().unwrap();
        assert_eq!(streaming.limit, Some(1000));
        assert_eq!(streaming.interval, Some(5));
        assert_eq!(streaming.quantity, Some(10));
    }

    #[test]
    fn test_parse_with_affiliate() {
        let memo = "=:ETH:0x789::::affiliate:250";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        let affiliate = parsed.affiliate.as_ref().unwrap();
        assert_eq!(affiliate.address, "affiliate");
        assert_eq!(affiliate.fee_bps, 250);
    }

    #[test]
    fn test_parse_empty_streaming_params() {
        let memo = "=:ETH:0x789:///:affiliate:50";
        let parsed = ThorchainMemo::parse(memo).unwrap();

        let streaming = parsed.streaming_swap.as_ref().unwrap();
        assert_eq!(streaming.limit, None);
        assert_eq!(streaming.interval, None);
        assert_eq!(streaming.quantity, None);
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
