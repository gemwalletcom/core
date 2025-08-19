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
        
        Ok(staking_mapper::map_staking_validators(vote_accounts.current, self.get_chain(), apy.unwrap_or(inflation_rate.validator * 100.0)))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let epoch = self.get_epoch_info().await?;
        let stake_accounts = self.get_staking_balance(&address).await?;

        Ok(staking_mapper::map_staking_delegations(
            stake_accounts,
            epoch,
            self.get_chain().as_asset_id(),
        ))
    }
}


