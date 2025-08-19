use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::{models::balance::HypercoreValidator, provider::staking_mapper, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainStaking for HyperCoreClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_validators().await?;
        let apy = HypercoreValidator::max_apr(validators);
        Ok(Some(apy))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Send + Sync>> {
        let validators = self.get_validators().await?;
        Ok(staking_mapper::map_validators_to_delegation_validators(validators, self.chain, apy))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_staking_delegations(&address).await?;
        Ok(staking_mapper::map_delegations_to_delegation_bases(delegations, self.chain))
    }
}
