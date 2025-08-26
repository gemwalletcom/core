use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_balance_coin, map_balance_tokens};
use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainBalances for AlgorandClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(map_balance_coin(&account.account, self.get_chain()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(map_balance_tokens(&account.account, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
mod integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainBalances;
    use primitives::Chain;

    #[tokio::test]
    async fn test_algorand_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Algorand);
        println!("Balance: {:?}", balance);
        assert!(balance.balance.available.parse::<u64>().unwrap() > 0);
        Ok(())
    }
}
