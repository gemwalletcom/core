use async_trait::async_trait;
use chain_traits::ChainState;
use primitives::{FeePriority, FeeRate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainState for HyperCoreClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("1".to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(1)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::regular(FeePriority::Normal, 1)])
    }
}
