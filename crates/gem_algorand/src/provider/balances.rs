use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_balance_coin, map_balance_tokens};
use crate::{rpc::client::AlgorandClient, AlgorandClientIndexer};

#[async_trait]
impl<C: Client> ChainBalances for AlgorandClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(map_balance_coin(&account, self.get_chain()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(map_balance_tokens(&account, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[async_trait]
impl<C: Client> ChainBalances for AlgorandClientIndexer<C> {
    async fn get_balance_coin(&self, _address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        unimplemented!()
    }
}

#[cfg(all(test, feature = "integration_tests"))]
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

    #[tokio::test]
    async fn test_algorand_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let token_ids = vec![
            "31566704".to_string(), // USDC
        ];
        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids).await?;

        assert_eq!(balances.len(), 1);
        for balance in &balances {
            assert_eq!(balance.asset_id.chain, Chain::Algorand);

            println!("Token balance: {:?}", balance);
            assert!(balance.balance.available.parse::<u64>().unwrap() > 0);
        }
        Ok(())
    }
}
