use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::{
    provider::staking_mapper::{calculate_network_apy, map_staking_delegations, map_staking_validators},
    rpc::client::SolanaClient,
};

#[async_trait]
impl<C: Client + Clone> ChainStaking for SolanaClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let (inflation_rate, supply, accounts) = futures::try_join!(self.get_inflation_rate(), self.get_supply(), self.get_vote_accounts(false))?;
        let total_active_stake = accounts.current.iter().map(|validator| validator.activated_stake).sum();
        Ok(Some(calculate_network_apy(inflation_rate.validator, supply.value.total, total_active_stake)))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_vote_accounts(false).await?;
        let network_apy = match apy {
            Some(apy) => apy,
            None => {
                let (inflation_rate, supply) = futures::try_join!(self.get_inflation_rate(), self.get_supply())?;
                let total_active_stake = accounts.current.iter().map(|validator| validator.activated_stake).sum();
                calculate_network_apy(inflation_rate.validator, supply.value.total, total_active_stake)
            }
        };
        Ok(map_staking_validators(accounts.current, self.get_chain(), network_apy))
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
        let apy = client.get_staking_apy().await?.unwrap();

        assert!(apy > 0.0);
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
