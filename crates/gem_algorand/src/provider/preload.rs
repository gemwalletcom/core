use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for AlgorandClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, _input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let params = self.get_transactions_params().await?;
        let metadata = TransactionLoadMetadata::Algorand {
            sequence: params.last_round,
            block_hash: params.genesis_hash,
            chain_id: params.genesis_id,
        };

        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_fee(BigInt::from(params.min_fee)),
            metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}
