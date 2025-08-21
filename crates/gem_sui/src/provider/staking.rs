#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainStaking;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::{DelegationBase, DelegationValidator};
#[cfg(feature = "rpc")]
use std::error::Error;

#[cfg(feature = "rpc")]
use super::staking_mapper;
#[cfg(feature = "rpc")]
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainStaking for SuiClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_validators().await?;
        let max_apy = validators.apys.iter().map(|v| v.apy).fold(0.0, f64::max);
        Ok(Some(max_apy))
    }

    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_validators().await?;
        Ok(staking_mapper::map_validators(validators, apy.unwrap_or(0.0)))
    }

    async fn get_staking_delegations(&self, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let system_state = self.get_system_state().await?;
        let delegations = self.get_stake_delegations(address).await?;
        Ok(staking_mapper::map_delegations(delegations, system_state))
    }
}
