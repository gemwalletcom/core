use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
};

use crate::{provider::preload_mapper::map_transaction_preload, rpc::client::XRPClient};

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for XRPClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        let result = self.get_account_info_full(&input.sender_address).await?;
        map_transaction_preload(result)
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
            FeeRate::new(
                FeePriority::Slow,
                GasPriceType::regular(BigInt::from(std::cmp::max(minimum_fee, median_fee / 2))),
            ),
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(median_fee))),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(median_fee * 2))),
        ])
    }
}
