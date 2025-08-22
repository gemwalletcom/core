use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{provider::preload_mapper::map_transaction_preload, rpc::client::XRPClient};

#[async_trait]
impl<C: Client> ChainTransactionLoad for XRPClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        let account_result = self.get_account_info_full(&input.sender_address).await?;
        map_transaction_preload(account_result)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
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
