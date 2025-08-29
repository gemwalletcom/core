use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use super::staking_mapper::map_staking_validators;
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainStaking for TronClient<C> {
    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let witnesses = self.get_witnesses_list().await?;
        Ok(map_staking_validators(witnesses, apy))
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
