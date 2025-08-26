use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use crate::provider::balances_mapper;
use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainBalances for PolkadotClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(address).await?;
        Ok(balances_mapper::map_coin_balance(balance))
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
    use super::*;
    use crate::provider::testkit::{create_polkadot_test_client, TEST_ADDRESS};

    #[tokio::test]
    async fn test_polkadot_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let address = TEST_ADDRESS.to_string();
        let balance = client.get_balance_coin(address).await?;
        assert_eq!(balance.asset_id.chain.to_string(), "polkadot");
        println!("Balance: {:?} {}", balance.balance, balance.asset_id);
        Ok(())
    }
}
