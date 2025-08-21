use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures;
use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::provider::preload_mapper::map_transaction_preload;
use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for BitcoinClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let utxos = self.get_utxos(&input.sender_address).await?;
        Ok(map_transaction_preload(utxos, input))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let priority = self.chain.get_blocks_fee_priority();
        let (slow, normal, fast) = futures::try_join!(self.get_fee(priority.slow), self.get_fee(priority.normal), self.get_fee(priority.fast))?;
        Ok(vec![
            FeeRate::regular(FeePriority::Slow, slow),
            FeeRate::regular(FeePriority::Normal, normal),
            FeeRate::regular(FeePriority::Fast, fast),
        ])
    }
}

impl<C: Client> BitcoinClient<C> {
    async fn get_fee(&self, blocks: i32) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
        let fee_sat_per_kb = self.get_fee_priority(blocks).await?;
        calculate_fee_rate(&fee_sat_per_kb, self.chain.minimum_byte_fee() as u32)
    }
}

fn calculate_fee_rate(fee_sat_per_kb: &str, minimum_byte_fee: u32) -> Result<BigInt, Box<dyn Error + Sync + Send>> {
    let rate = BigNumberFormatter::value_from_amount(fee_sat_per_kb, 8)?.parse::<f64>()? / 1000.0;
    let minimum_byte_fee = minimum_byte_fee as f64;

    Ok(BigInt::from(rate.max(minimum_byte_fee) as i64))
}
