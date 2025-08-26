use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper;
use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainBalances for NearClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        balances_mapper::map_native_balance(&account)
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{create_near_test_client, TEST_ADDRESS};
    use chain_traits::ChainBalances;

    #[tokio::test]
    async fn test_near_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_near_test_client()?;
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;
        let available = balance.balance.available.parse::<u128>();
        assert!(available.is_ok());
        println!("Balance: {} {}", balance.balance.available, balance.asset_id);
        Ok(())
    }
}
