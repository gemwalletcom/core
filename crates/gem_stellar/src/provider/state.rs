use async_trait::async_trait;
use chain_traits::ChainState;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainState for StellarClient<C> {
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.ingest_latest_ledger as u64)
    }

    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.network_passphrase)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;

        let min_fee = std::cmp::max(
            fees.fee_charged.min.parse::<u64>().unwrap_or(100),
            fees.last_ledger_base_fee.parse::<u64>().unwrap_or(100),
        );

        let fast_fee = fees.fee_charged.p95.parse::<u64>().unwrap_or(min_fee) * 2;

        Ok(vec![
            FeeRate::regular(FeePriority::Slow, BigInt::from(min_fee)),
            FeeRate::regular(FeePriority::Normal, BigInt::from(min_fee)),
            FeeRate::regular(FeePriority::Fast, BigInt::from(fast_fee)),
        ])
    }
}
