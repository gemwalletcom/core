use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriorityValue, FeePriority};

use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainState for AlgorandClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transactions_params().await?.genesis_id)
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transactions_params().await?.last_round as u64)
    }

    async fn get_fees(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeePriorityValue {
            priority: FeePriority::Normal,
            value: self.get_transactions_params().await?.min_fee.to_string(),
        }])
    }
}