use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeePriorityValue};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainState for StellarClient<C> {
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.ingest_latest_ledger as u64)
    }

    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.network_passphrase)
    }

    async fn get_fees(&self) -> Result<Vec<primitives::FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;

        let min_fee = std::cmp::max(
            fees.fee_charged.min.parse::<u64>().unwrap_or(100),
            fees.last_ledger_base_fee.parse::<u64>().unwrap_or(100),
        );

        let fast_fee = fees.fee_charged.p95.parse::<u64>().unwrap_or(min_fee) * 2;

        Ok(vec![
            FeePriorityValue::new(FeePriority::Slow, min_fee.to_string()),
            FeePriorityValue::new(FeePriority::Normal, min_fee.to_string()),
            FeePriorityValue::new(FeePriority::Fast, fast_fee.to_string()),
        ])
    }
}
