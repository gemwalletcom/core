use async_trait::async_trait;
use chain_traits::ChainStaking;
use futures::try_join;
use std::error::Error;

use gem_client::Client;
use primitives::{chain_cosmos::CosmosChain, DelegationBase, DelegationValidator};

use crate::{
    provider::staking_mapper::{calculate_network_apy_cosmos, calculate_network_apy_osmosis, map_staking_delegations, map_staking_validators},
    rpc::client::CosmosClient,
};

#[async_trait]
impl<C: Client> ChainStaking for CosmosClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        match chain {
            CosmosChain::Noble | CosmosChain::Thorchain => Ok(None),
            CosmosChain::Cosmos | CosmosChain::Injective => {
                let (inflation, staking_pool) = try_join!(self.get_inflation(), self.get_staking_pool())?;
                Ok(calculate_network_apy_cosmos(inflation, staking_pool))
            }
            CosmosChain::Osmosis => {
                let (mint_params, epoch_provisions, staking_pool, supply) = try_join!(
                    self.get_osmosis_mint_params(),
                    self.get_osmosis_epoch_provisions(),
                    self.get_staking_pool(),
                    self.get_supply_by_denom("uosmo")
                )?;

                Ok(calculate_network_apy_osmosis(mint_params, epoch_provisions, staking_pool, supply))
            }
            CosmosChain::Celestia => Ok(Some(10.55)),
            CosmosChain::Sei => Ok(Some(5.62)),
        }
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain().as_chain();
        if !chain.is_stake_supported() {
            return Ok(vec![]);
        }

        let validators = self.get_validators().await?;

        Ok(map_staking_validators(validators.validators, chain, apy))
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

        Ok(map_staking_delegations(
            active_delegations,
            unbonding,
            rewards,
            validators.validators,
            chain,
            denom,
        ))
    }
}
