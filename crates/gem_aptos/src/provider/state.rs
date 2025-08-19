use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeePriorityValue};

use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainState for AptosClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.chain_id.to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.ledger_version)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        let gas_fee = self.get_gas_price().await?;

        Ok(vec![
            FeePriorityValue {
                priority: FeePriority::Slow,
                value: gas_fee.deprioritized_gas_estimate.to_string(),
            },
            FeePriorityValue {
                priority: FeePriority::Normal,
                value: gas_fee.gas_estimate.to_string(),
            },
            FeePriorityValue {
                priority: FeePriority::Fast,
                value: gas_fee.prioritized_gas_estimate.to_string(),
            },
        ])
    }
}
