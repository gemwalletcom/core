use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::{provider::staking_mapper, rpc::client::CosmosClient};

#[async_trait]
impl<C: Client> ChainStaking for CosmosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let cosmos_chain = self.get_chain();
        if !cosmos_chain.as_chain().is_stake_supported() {
            return Ok(None);
        }

        let (inflation, staking_pool) = try_join!(self.get_inflation(), self.get_staking_pool())?;

        let inflation_rate = inflation.inflation.parse::<f64>().unwrap_or(0.0);
        let bonded_tokens = staking_pool.pool.bonded_tokens.parse::<f64>().unwrap_or(1.0);
        let total_supply = bonded_tokens + staking_pool.pool.not_bonded_tokens.parse::<f64>().unwrap_or(0.0);

        Ok(staking_mapper::calculate_network_apy(cosmos_chain, inflation_rate, bonded_tokens, total_supply))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain().as_chain();
        if !chain.is_stake_supported() {
            return Ok(vec![]);
        }

        let validators = self.get_validators().await?;

        Ok(staking_mapper::map_staking_validators(validators.validators, chain, apy))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain().as_chain();
        if !chain.is_stake_supported() {
            return Ok(vec![]);
        }

        let denom = chain.as_denom().unwrap_or_default();

        let (active_delegations, unbonding, rewards) = try_join!(
            self.get_delegations(&address),
            self.get_unbonding_delegations(&address),
            self.get_delegation_rewards(&address)
        )?;

        let validators = self.get_validators().await?;

        Ok(staking_mapper::map_staking_delegations(
            active_delegations,
            unbonding,
            rewards,
            validators.validators,
            chain,
            denom,
        ))
    }
}
