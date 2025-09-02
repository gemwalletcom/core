use std::error::Error;

use crate::provider::token_mapper::{map_is_token_address, map_token_data};
use crate::rpc::client::{EthereumClient, FUNCTION_ERC20_DECIMALS, FUNCTION_ERC20_NAME, FUNCTION_ERC20_SYMBOL};

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainToken;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::Asset;
#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainToken for EthereumClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let [name, symbol, decimals] = self
            .batch_eth_call(&token_id, [FUNCTION_ERC20_NAME, FUNCTION_ERC20_SYMBOL, FUNCTION_ERC20_DECIMALS])
            .await?;

        map_token_data(self.get_chain(), token_id, name, symbol, decimals)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        map_is_token_address(token_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client};
    use primitives::{AssetType, Chain};

    #[tokio::test]
    async fn test_ethereum_get_token_data_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();

        let asset = client.get_token_data(usdc_address.clone()).await?;

        println!("USDC Asset: asset={:?}", asset);

        assert_eq!(asset.name, "USD Coin");
        assert_eq!(asset.symbol, "USDC");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.id.chain, Chain::Ethereum);
        assert_eq!(asset.id.token_id, Some(usdc_address));
        assert_eq!(asset.asset_type, AssetType::ERC20);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_token_data_usdt() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let usdt_address = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();

        let asset = client.get_token_data(usdt_address.clone()).await?;

        println!("USDT Asset: asset={:?}", asset);

        assert_eq!(asset.name, "Tether USD");
        assert_eq!(asset.symbol, "USDT");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.id.chain, Chain::Ethereum);
        assert_eq!(asset.id.token_id, Some(usdt_address));
        assert_eq!(asset.asset_type, AssetType::ERC20);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_token_data_usdt() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let usdt_address = "0x55d398326f99059fF775485246999027B3197955".to_string();

        let asset = client.get_token_data(usdt_address.clone()).await?;

        println!("BSC USDT Asset: asset={:?}", asset);

        assert_eq!(asset.name, "Tether USD");
        assert_eq!(asset.symbol, "USDT");
        assert_eq!(asset.decimals, 18);
        assert_eq!(asset.id.chain, Chain::SmartChain);
        assert_eq!(asset.id.token_id, Some(usdt_address));
        assert_eq!(asset.asset_type, AssetType::BEP20);

        Ok(())
    }
}
