use primitives::{Asset, AssetId};

use super::chain::THORChainName;

#[derive(Clone)]
pub struct THORChainAsset {
    pub symbol: String,
    pub chain: THORChainName,
    pub token_id: Option<String>,
    pub decimals: u32,
}

impl THORChainAsset {
    pub fn asset_name(&self) -> String {
        if self.token_id.is_some() {
            format!("{}.{}", self.chain.long_name(), self.symbol)
        } else {
            self.chain.short_name().to_string()
        }
    }

    pub fn from_asset_id(asset_id: AssetId) -> Option<THORChainAsset> {
        let chain = THORChainName::from_chain(&asset_id.chain)?;
        if let Some(token_id) = &asset_id.token_id {
            THORChainAsset::from(chain, token_id)
        } else {
            let asset = Asset::from_chain(asset_id.chain);
            Some(THORChainAsset {
                symbol: asset.symbol,
                chain,
                token_id: None,
                decimals: asset.decimals as u32,
            })
        }
    }

    pub fn from(chain: THORChainName, token_id: &str) -> Option<THORChainAsset> {
        match chain {
            THORChainName::Ethereum => match token_id {
                "0xdAC17F958D2ee523a2206206994597C13D831ec7" => Some(THORChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 6,
                }),
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" => Some(THORChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 6,
                }),
                "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599" => Some(THORChainAsset {
                    symbol: "WBTC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 8,
                }),
                "0x6B175474E89094C44Da98b954EedeAC495271d0F" => Some(THORChainAsset {
                    symbol: "DAI".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            THORChainName::SmartChain => match token_id {
                "0x55d398326f99059fF775485246999027B3197955" => Some(THORChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d" => Some(THORChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            _ => None,
        }
    }

    // https://dev.thorchain.org/concepts/memos.html#swap
    pub fn get_memo(&self, destination_address: String, fee_address: String, bps: u32) -> Option<String> {
        Some(format!("=:{}:{}::{}:{}", self.asset_name(), destination_address, fee_address, bps))
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_thorchain_name_token() {
        let test_cases = vec![
            ("0xdAC17F958D2ee523a2206206994597C13D831ec7", THORChainName::Ethereum, "USDT", 6),
            ("0x55d398326f99059fF775485246999027B3197955", THORChainName::SmartChain, "USDT", 18),
        ];

        for (token_id, chain, expected_symbol, expected_decimals) in test_cases {
            let asset = THORChainAsset::from(chain, token_id);
            assert!(asset.is_some());
            let asset = asset.unwrap();
            assert_eq!(asset.symbol, expected_symbol);
            assert_eq!(asset.decimals, expected_decimals);
        }
    }

    #[test]
    fn test_thorchain_asset_name() {
        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: THORChainName::Ethereum,
            token_id: Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.asset_name(), "ETH.USDT");

        let asset_with_token = THORChainAsset {
            symbol: "USDT".to_string(),
            chain: THORChainName::SmartChain,
            token_id: Some("0x55d398326f99059fF775485246999027B3197955".to_string()),
            decimals: 6,
        };
        assert_eq!(asset_with_token.asset_name(), "BSC.USDT");

        let asset_without_token = THORChainAsset {
            symbol: "RUNE".to_string(),
            chain: THORChainName::Thorchain,
            token_id: None,
            decimals: 8,
        };
        assert_eq!(asset_without_token.asset_name(), "r");
    }

    #[test]
    fn test_get_memo() {
        let destination_address = "0x1234567890abcdef".to_string();
        let fee_address = "1".to_string();
        let bps = 50;

        assert_eq!(
            THORChainAsset::from_asset_id(Chain::SmartChain.as_asset_id())
                .unwrap()
                .get_memo(destination_address.clone(), fee_address.clone(), bps),
            Some("=:s:0x1234567890abcdef::1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Ethereum.as_asset_id())
                .unwrap()
                .get_memo(destination_address.clone(), fee_address.clone(), bps),
            Some("=:e:0x1234567890abcdef::1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Doge.as_asset_id())
                .unwrap()
                .get_memo(destination_address.clone(), fee_address.clone(), bps),
            Some("=:d:0x1234567890abcdef::1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(AssetId::from(Chain::Ethereum, Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string())))
                .unwrap()
                .get_memo(destination_address.clone(), fee_address.clone(), bps),
            Some("=:ETH.USDT:0x1234567890abcdef::1:50".into())
        );
    }
}
