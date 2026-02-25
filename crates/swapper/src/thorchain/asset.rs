use num_bigint::BigInt;
use primitives::{Asset, AssetId};
use std::str::FromStr;

use crate::asset::*;

use super::chain::THORChainName;

const THORCHAIN_DECIMALS: i32 = 8;

pub fn value_from(value: &str, decimals: i32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let diff = decimals - THORCHAIN_DECIMALS;
    let factor = BigInt::from(10).pow(diff.unsigned_abs());
    if diff > 0 { value / factor } else { value * factor }
}

pub fn value_to(value: &str, decimals: i32) -> BigInt {
    let value = BigInt::from_str(value).unwrap_or_default();
    let diff = decimals - THORCHAIN_DECIMALS;
    let factor = BigInt::from(10).pow(diff.unsigned_abs());
    if diff > 0 { value * factor } else { value / factor }
}

#[derive(Clone, Debug)]
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

    pub fn is_token(&self) -> bool {
        self.token_id.is_some()
    }

    pub fn use_evm_router(&self) -> bool {
        self.is_token() && self.chain.is_evm_chain()
    }

    pub fn from_asset_id(asset_id: &str) -> Option<THORChainAsset> {
        let asset_id = AssetId::new(asset_id)?;
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

    //TODO: Refactor to remove mapping.
    pub fn from(chain: THORChainName, token_id: &str) -> Option<THORChainAsset> {
        match chain {
            THORChainName::Ethereum => match token_id {
                ETHEREUM_USDT_TOKEN_ID => Some(chain.asset(ETHEREUM_USDT.clone())),
                ETHEREUM_USDC_TOKEN_ID => Some(chain.asset(ETHEREUM_USDC.clone())),
                ETHEREUM_WBTC_TOKEN_ID => Some(chain.asset(ETHEREUM_WBTC.clone())),
                ETHEREUM_DAI_TOKEN_ID => Some(chain.asset(ETHEREUM_DAI.clone())),
                _ => None,
            },
            THORChainName::SmartChain => match token_id {
                SMARTCHAIN_USDT_TOKEN_ID => Some(chain.asset(SMARTCHAIN_USDT.clone())),
                SMARTCHAIN_USDC_TOKEN_ID => Some(chain.asset(SMARTCHAIN_USDC.clone())),
                _ => None,
            },
            THORChainName::AvalancheC => match token_id {
                AVALANCHE_USDT_TOKEN_ID => Some(chain.asset(AVALANCHE_USDT.clone())),
                AVALANCHE_USDC_TOKEN_ID => Some(chain.asset(AVALANCHE_USDC.clone())),
                _ => None,
            },
            THORChainName::Base => match token_id {
                BASE_USDC_TOKEN_ID => Some(chain.asset(BASE_USDC.clone())),
                BASE_CBBTC_TOKEN_ID => Some(chain.asset(BASE_CBBTC.clone())),
                _ => None,
            },
            THORChainName::Thorchain => match token_id {
                THORCHAIN_TCY_TOKEN_ID => Some(chain.asset(THORCHAIN_TCY.clone())),
                _ => None,
            },
            THORChainName::Tron => match token_id {
                TRON_USDT_TOKEN_ID => Some(chain.asset(TRON_USDT.clone())),
                _ => None,
            },
            THORChainName::Solana => match token_id {
                SOLANA_USDC_TOKEN_ID => Some(chain.asset(SOLANA_USDC.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    // https://dev.thorchain.org/concepts/memos.html#swap
    pub fn get_memo(&self, destination_address: String, minimum: i64, interval: i64, quantity: i64, fee_address: String, bps: u32) -> Option<String> {
        let address = match self.chain {
            THORChainName::BitcoinCash => destination_address.strip_prefix("bitcoincash:").unwrap_or(&destination_address),
            _ => &destination_address,
        };
        Some(format!("=:{}:{}:{}/{}/{}:{}:{}", self.asset_name(), address, minimum, interval, quantity, fee_address, bps))
    }
}

impl THORChainName {
    pub fn asset(&self, asset: Asset) -> THORChainAsset {
        THORChainAsset {
            symbol: asset.symbol,
            chain: self.clone(),
            token_id: asset.id.token_id,
            decimals: asset.decimals as u32,
        }
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
        let fee_address = "g1".to_string();
        let bps = 50;

        assert_eq!(
            THORChainAsset::from_asset_id(Chain::SmartChain.as_ref())
                .unwrap()
                .get_memo(destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            Some("=:s:0x1234567890abcdef:0/1/0:g1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Ethereum.as_ref())
                .unwrap()
                .get_memo(destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            Some("=:e:0x1234567890abcdef:0/1/0:g1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::Doge.as_ref())
                .unwrap()
                .get_memo(destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            Some("=:d:0x1234567890abcdef:0/1/0:g1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id("ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7")
                .unwrap()
                .get_memo(destination_address.clone(), 0, 1, 0, fee_address.clone(), bps),
            Some("=:ETH.USDT:0x1234567890abcdef:0/1/0:g1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(Chain::BitcoinCash.as_ref()).unwrap().get_memo(
                "bitcoincash:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65".to_string(),
                0,
                1,
                0,
                fee_address.clone(),
                bps
            ),
            Some("=:c:qpcns7lget89x9km0t8ry5fk52e8lhl53q0a64gd65:0/1/0:g1:50".into())
        );
        assert_eq!(
            THORChainAsset::from_asset_id(&AssetId::from_token(Chain::Thorchain, "tcy").to_string()).unwrap().get_memo(
                destination_address.clone(),
                0,
                1,
                0,
                fee_address.clone(),
                bps
            ),
            Some("=:THOR.TCY:0x1234567890abcdef:0/1/0:g1:50".into())
        );
    }

    #[test]
    fn test_solana_usdc_memo() {
        let destination = "HN7cABqLq46Es1jh92dQQisAi5YqpaAqUoM3pMYmZYBN".to_string();
        let fee_address = "g1".to_string();
        let bps = 50;

        let solana_usdc_asset_id = &AssetId::from_token(Chain::Solana, SOLANA_USDC_TOKEN_ID).to_string();
        let asset = THORChainAsset::from_asset_id(solana_usdc_asset_id);

        assert!(asset.is_some());

        let memo = asset.unwrap().get_memo(destination, 0, 1, 0, fee_address, bps);

        assert_eq!(memo, Some("=:SOL.USDC:HN7cABqLq46Es1jh92dQQisAi5YqpaAqUoM3pMYmZYBN:0/1/0:g1:50".into()));
    }

    #[test]
    fn test_value_from() {
        assert_eq!(value_from("1000000000000000000", 18), BigInt::from(100000000));
        assert_eq!(value_from("1000000000", 10), BigInt::from(10000000));
        assert_eq!(value_from("1000000000", 6), BigInt::from_str("100000000000").unwrap());
        assert_eq!(value_from("1000000000", 8), BigInt::from(1000000000));
    }

    #[test]
    fn test_value_to() {
        assert_eq!(value_to("2509674", 18), BigInt::from_str("25096740000000000").unwrap());
        assert_eq!(value_to("10000000", 10), BigInt::from(1000000000));
        assert_eq!(value_to("79158429", 6), BigInt::from(791584));
        assert_eq!(value_to("160661010", 8), BigInt::from(160661010));
    }

    #[test]
    fn test_tron_usdt_memo() {
        let tron_destination = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string();
        let fee_address = "g1".to_string();
        let bps = 50;

        let tron_usdt_asset_id = &AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t").to_string();
        let asset = THORChainAsset::from_asset_id(tron_usdt_asset_id);

        assert!(asset.is_some(), "TRON USDT asset should be recognized");

        let memo = asset.unwrap().get_memo(tron_destination.clone(), 0, 1, 0, fee_address.clone(), bps);

        assert_eq!(memo, Some("=:TRON.USDT:TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb:0/1/0:g1:50".into()));
    }
}
