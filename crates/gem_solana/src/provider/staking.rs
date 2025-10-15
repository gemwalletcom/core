use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator, StakeChain, StakeLockTime};

const SOLANA_ACTIVATION_LOCK_TIME: u64 = 259_200;

use crate::{
    provider::staking_mapper::{map_staking_delegations, map_staking_validators},
    rpc::client::SolanaClient,
};

#[async_trait]
impl<C: Client + Clone> ChainStaking for SolanaClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(self.get_inflation_rate().await?.validator * 100.0))
    }

    async fn get_staking_lock_time(&self) -> Result<StakeLockTime, Box<dyn Error + Sync + Send>> {
        Ok(StakeLockTime::new(StakeChain::Solana.get_lock_time(), Some(SOLANA_ACTIVATION_LOCK_TIME)))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let (accounts, inflation_rate) = futures::try_join!(self.get_vote_accounts(false), self.get_inflation_rate())?;
        Ok(map_staking_validators(
            accounts.current,
            self.get_chain(),
            apy.unwrap_or(inflation_rate.validator * 100.0),
        ))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let (epoch, accounts) = futures::try_join!(self.get_epoch_info(), self.get_staking_balance(&address))?;
        Ok(map_staking_delegations(accounts, epoch, self.get_chain().as_asset_id()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_solana_test_client};

    #[tokio::test]
    async fn test_solana_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let apy = client.get_staking_apy().await?;
        assert!(apy.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let validators = client.get_staking_validators(None).await?;
        assert!(!validators.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let delegations = client.get_staking_delegations(TEST_ADDRESS.to_string()).await?;
        assert!(delegations.len() <= 100);
        Ok(())
    }
}
