use async_trait::async_trait;
use chain_traits::ChainState;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate};

use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainState for TronClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("".to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_latest_block().await? as u64)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::regular(FeePriority::Normal, BigInt::from(1))])
    }
}
