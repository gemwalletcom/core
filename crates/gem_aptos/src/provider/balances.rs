use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_native_balance, map_token_balances};
use crate::rpc::client::AptosClient;
use crate::APTOS_NATIVE_COIN;

#[async_trait]
impl<C: Client> ChainBalances for AptosClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_account_balance(&address, APTOS_NATIVE_COIN).await?;
        Ok(map_native_balance(&balance.to_string(), self.get_chain()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let resources = self.get_account_resources(&address).await?;
        Ok(map_token_balances(&resources, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{create_aptos_test_client, TEST_ADDRESS};
    use chain_traits::ChainBalances;
    use primitives::Chain;

    #[tokio::test]
    async fn test_aptos_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Aptos);
        println!("Balance: {:?}", balance);
        Ok(())
    }
}
