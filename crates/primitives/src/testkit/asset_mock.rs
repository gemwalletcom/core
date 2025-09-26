use crate::{Asset, AssetId, AssetType, Chain};

impl Asset {
    pub fn mock() -> Self {
        Asset::from_chain(Chain::Ethereum)
    }

    pub fn mock_sol() -> Self {
        Asset::from_chain(Chain::Solana)
    }

    pub fn mock_spl_token() -> Self {
        Asset::new(
            AssetId::from_token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "USD Coin".to_string(),
            "USDC".to_string(),
            6,
            AssetType::SPL,
        )
    }

    pub fn mock_ethereum_usdc() -> Self {
        Asset::new(
            AssetId::from_token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            "USD Coin".to_string(),
            "USDC".to_string(),
            6,
            AssetType::ERC20,
        )
    }

    pub fn mock_eth() -> Self {
        Asset::from_chain(Chain::Ethereum)
    }

    pub fn mock_btc() -> Self {
        Asset::from_chain(Chain::Bitcoin)
    }

    pub fn mock_erc20() -> Self {
        Asset::new(
            AssetId::from_token(Chain::Ethereum, "0xA0b86a33E6441066d64bb38954e41F6b4b925c59"),
            "USD Coin".to_string(),
            "USDC".to_string(),
            6,
            AssetType::ERC20,
        )
    }

    pub fn mock_with_chain(chain: Chain) -> Self {
        Asset::from_chain(chain)
    }

    pub fn mock_with_params(chain: Chain, token_id: Option<String>, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Self {
        Asset::new(AssetId::from(chain, token_id), name, symbol, decimals, asset_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_mock() {
        let asset = Asset::mock();
        assert_eq!(asset.symbol, "ETH");
        assert_eq!(asset.chain, Chain::Ethereum);

        let sol_asset = Asset::mock_sol();
        assert_eq!(sol_asset.symbol, "SOL");
        assert_eq!(sol_asset.chain, Chain::Solana);

        let spl_asset = Asset::mock_spl_token();
        assert_eq!(spl_asset.symbol, "USDC");
        assert_eq!(spl_asset.asset_type, AssetType::SPL);
    }
}
