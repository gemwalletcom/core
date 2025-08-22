use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        let block = self.get_tron_block().await?.block_header.raw_data;

        Ok(TransactionLoadMetadata::Tron {
            block_number: block.number,
            block_version: block.version as u64,
            block_timestamp: block.timestamp,
            transaction_tree_root: block.tx_trie_root.clone(),
            parent_hash: block.parent_hash.clone(),
            witness_address: block.witness_address.clone(),
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_fee(input.gas_price.total_fee()),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}
