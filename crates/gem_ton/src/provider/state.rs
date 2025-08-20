use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;
use num_bigint::BigInt;

use gem_client::Client;
use primitives::{FeePriority, FeeRate};

use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainState for TonClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let master_head = self.get_master_head().await?;
        Ok(master_head.initial.root_hash)
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_latest_block().await? as u64)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![
            FeeRate::regular(FeePriority::Normal, BigInt::from(10000000)), // 0.01 TON
        ])
    }
}
