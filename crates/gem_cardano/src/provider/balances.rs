use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::map_balance_coin;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainBalances for CardanoClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(map_balance_coin(balance, self.get_chain()))
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use num_bigint::BigUint;

    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_test_client};

    #[tokio::test]
    async fn test_cardano_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;

        println!("Balance: {:?} {}", balance.balance.available, balance.asset_id);

        assert!(balance.balance.available > BigUint::from(0u64));
        Ok(())
    }

    #[tokio::test]
    async fn test_cardano_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let token_ids = vec![];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_cardano_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        assert!(balance.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_cardano_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let assets = client.get_balance_assets(TEST_ADDRESS.to_string()).await?;
        assert!(assets.is_empty());
        Ok(())
    }
}
