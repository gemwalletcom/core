use async_trait::async_trait;
use chain_traits::ChainBalances;
use num_bigint::BigUint;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetBalance, AssetId};

use super::balances_mapper::{map_balance_coin, map_balance_tokens};
use crate::rpc::client::AlgorandClient;

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

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.get_account(&address).await?;
        let asset_balances: Vec<AssetBalance> = account
            .assets
            .into_iter()
            .map(|asset| AssetBalance::new(AssetId::from(self.get_chain(), Some(asset.asset_id.to_string())), BigUint::from(asset.amount)))
            .collect();

        Ok(asset_balances)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainBalances;
    use primitives::Chain;

    #[tokio::test]
    async fn test_algorand_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;
        assert_eq!(balance.asset_id.chain, Chain::Algorand);
        println!("Balance: {:?}", balance);
        assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
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
            assert!(balance.balance.available > num_bigint::BigUint::from(0u32));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_balance_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let address = TEST_ADDRESS.to_string();
        let assets = client.get_balance_assets(address).await?;

        assert!(assets.is_empty());

        for asset in &assets {
            assert_eq!(asset.asset_id.chain, primitives::Chain::Algorand);
            assert!(asset.balance.available > num_bigint::BigUint::from(0u32));
            assert!(asset.asset_id.token_id.is_some());
        }

        Ok(())
    }
}
