use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use super::preload_mapper::map_transaction_preload;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for AptosClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&input.sender_address).await?;
        map_transaction_preload(&account)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let gas_limit = self.calculate_gas_limit(&input).await?;
        let fee = TransactionFee::calculate(gas_limit, &input.gas_price);

        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_fee = self.get_gas_price().await?;

        Ok(vec![
            FeeRate::new(FeePriority::Slow, GasPriceType::regular(gas_fee.deprioritized_gas_estimate)),
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(gas_fee.gas_estimate)),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(gas_fee.prioritized_gas_estimate)),
        ])
    }
}
