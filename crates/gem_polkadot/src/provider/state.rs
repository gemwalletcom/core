use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeePriorityValue};

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainState for PolkadotClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_version().await?.chain)
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_block_header("head").await?.number)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        Ok(vec![
            FeePriorityValue::new(FeePriority::Normal, "1".to_string())
        ])
    }
}