use async_trait::async_trait;
use chain_traits::ChainBalances;
use std::error::Error;

use crate::provider::balances_mapper::{map_balance_staking, map_coin_balance, map_token_accounts};
use crate::rpc::client::SolanaClient;
use gem_client::Client;
use primitives::AssetBalance;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBalances for SolanaClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(map_coin_balance(&balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let mut results = Vec::new();
        for token_id in token_ids {
            let accounts = self.get_token_accounts_by_mint(&address, &token_id).await?;
            results.extend(map_token_accounts(&accounts, &token_id));
        }
        Ok(results)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_staking_balance(&address).await?;
        Ok(map_balance_staking(accounts))
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use primitives::Chain;

    #[tokio::test]
    async fn test_solana_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let balance = client.get_balance_coin(TEST_ADDRESS.to_string()).await?;

        assert_eq!(balance.asset_id.chain, Chain::Solana);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available.parse::<u64>().is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let token_ids = vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
        ];

        let balances = client.get_balance_tokens(TEST_ADDRESS.to_string(), token_ids.clone()).await?;

        assert_eq!(balances.len(), token_ids.len());
        for (i, balance) in balances.iter().enumerate() {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
            assert!(balance.balance.available.parse::<u64>().is_ok());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let staking_balance = client.get_balance_staking(TEST_ADDRESS.to_string()).await?;

        if let Some(balance) = staking_balance {
            assert_eq!(balance.asset_id.chain, Chain::Solana);
            assert_eq!(balance.asset_id.token_id, None);
            assert!(balance.balance.available.parse::<u64>().is_ok());
            assert!(balance.balance.staked.parse::<u64>().is_ok());
        }

        Ok(())
    }
}
