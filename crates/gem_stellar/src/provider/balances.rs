use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use gem_client::Client;
use primitives::AssetBalance;

use super::balances_mapper::{map_all_balances, map_native_balance, map_token_balances};
use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainBalances for StellarClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let account = self.get_stellar_account(&address).await?;

        map_native_balance(&account)
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let account = self.get_stellar_account(&address).await?;
        Ok(map_token_balances(&account, token_ids, self.get_chain()))
    }

    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let account = self.get_stellar_account(&address).await?;
        Ok(map_all_balances(self.get_chain(), account))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use primitives::Chain;

    #[tokio::test]
    async fn test_stellar_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Stellar);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let token_ids = vec![
            "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC".to_string(), // USDC token
        ];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Stellar);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available >= num_bigint::BigUint::from(0u32));
        }

        Ok(())
    }
}
