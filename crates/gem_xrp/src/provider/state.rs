use async_trait::async_trait;
use chain_traits::ChainState;
use num_bigint::BigInt;
use std::error::Error;

use crate::rpc::client::XRPClient;
use gem_client::Client;
use primitives::{FeePriority, FeeRate};

#[async_trait]
impl<C: Client> ChainState for XRPClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("".to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger_current().await?.ledger_current_index as u64)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;

        let minimum_fee = fees.drops.minimum_fee;
        let median_fee = fees.drops.median_fee;

        Ok(vec![
            FeeRate::regular(FeePriority::Slow, BigInt::from(std::cmp::max(minimum_fee, median_fee / 2))),
            FeeRate::regular(FeePriority::Normal, BigInt::from(median_fee)),
            FeeRate::regular(FeePriority::Fast, BigInt::from(median_fee * 2)),
        ])
    }
}
