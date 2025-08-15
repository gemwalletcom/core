use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::fee::{FeePriority, FeePriorityValue};
use std::error::Error;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainState for BitcoinClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let block = self.get_block_info(1).await?;
        block.previous_block_hash.ok_or_else(|| "Unable to get block hash".into())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let node_info = self.get_node_info().await?;
        Ok(node_info.blockbook.best_height)
    }

    async fn get_fees(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let priority = self.chain.get_blocks_fee_priority();
        let (slow, normal, fast) = futures::try_join!(self.get_fee(priority.slow), self.get_fee(priority.normal), self.get_fee(priority.fast))?;
        Ok(vec![
            FeePriorityValue::new(FeePriority::Slow, slow),
            FeePriorityValue::new(FeePriority::Normal, normal),
            FeePriorityValue::new(FeePriority::Fast, fast),
        ])
    }
}

impl<C: Client> BitcoinClient<C> {
    async fn get_fee(&self, blocks: i32) -> Result<String, Box<dyn Error + Sync + Send>> {
        let fee = self.get_fee_priority(blocks).await?;
        Ok(BigNumberFormatter::value_from_amount(&fee, 8).unwrap())
    }
}
