use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainStaking for PolkadotClient<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(10.0)) // Default APY for Polkadot
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
