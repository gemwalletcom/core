use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;
use primitives::{DelegationBase, DelegationValidator};

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainStaking for BitcoinClient<C> {
    async fn get_staking_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
