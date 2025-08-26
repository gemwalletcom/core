use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::{provider::staking_mapper, rpc::client::SolanaClient};

#[async_trait]
impl<C: Client + Clone> ChainStaking for SolanaClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(self.get_inflation_rate().await?.validator * 100.0))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let vote_accounts = self.get_vote_accounts().await?;
        let inflation_rate = self.get_inflation_rate().await?;

        Ok(staking_mapper::map_staking_validators(
            vote_accounts.current,
            self.get_chain(),
            apy.unwrap_or(inflation_rate.validator * 100.0),
        ))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let epoch = self.get_epoch_info().await?;
        let stake_accounts = self.get_staking_balance(&address).await?;

        Ok(staking_mapper::map_staking_delegations(stake_accounts, epoch, self.get_chain().as_asset_id()))
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};

    #[tokio::test]
    async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let apy = client.get_staking_apy().await?;
        assert!(apy.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let validators = client.get_staking_validators(None).await?;
        assert!(!validators.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS.to_string()).await?;
        assert!(delegations.len() <= 100);
        Ok(())
    }
}
