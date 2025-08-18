use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use crate::rpc::client::XRPClient;
use gem_client::Client;
use primitives::{FeePriority, FeePriorityValue};

#[async_trait]
impl<C: Client> ChainState for XRPClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("".to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger_current().await?.ledger_current_index as u64)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;

        let minimum_fee = fees.drops.minimum_fee;
        let median_fee = fees.drops.median_fee;

        Ok(vec![
            FeePriorityValue::new(FeePriority::Slow, std::cmp::max(minimum_fee, median_fee / 2).to_string()),
            FeePriorityValue::new(FeePriority::Normal, median_fee.to_string()),
            FeePriorityValue::new(FeePriority::Fast, (median_fee * 2).to_string()),
        ])
    }
}
