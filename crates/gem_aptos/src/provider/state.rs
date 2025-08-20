use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;
use num_bigint::BigInt;

use gem_client::Client;
use primitives::{FeePriority, FeeRate};

use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainState for AptosClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.chain_id.to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.ledger_version)
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_fee = self.get_gas_price().await?;

        Ok(vec![
            FeeRate::regular(FeePriority::Slow, BigInt::from(gas_fee.deprioritized_gas_estimate)),
            FeeRate::regular(FeePriority::Normal, BigInt::from(gas_fee.gas_estimate)),
            FeeRate::regular(FeePriority::Fast, BigInt::from(gas_fee.prioritized_gas_estimate)),
        ])
    }
}
