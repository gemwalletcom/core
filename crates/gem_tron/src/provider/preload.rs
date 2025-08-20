use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput, TransactionLoadData, TransactionLoadInput, TransactionFee};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainPreload for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Send + Sync>> {
        let latest_block = self.get_latest_block().await?;
        
        Ok(TransactionPreload::builder()
            .block_number(latest_block)
            .build())
    }

    async fn get_transaction_load(&self, _input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let block = self.get_tron_block().await?;
        let block_data = &block.block_header.raw_data;
        
        let metadata = TransactionLoadMetadata::Tron {
            block_number: block_data.number,
            block_version: block_data.version,
            block_timestamp: block_data.timestamp,
            transaction_tree_root: block_data.tx_trie_root.clone(),
            parent_hash: block_data.parent_hash.clone(),
            witness_address: block_data.witness_address.clone(),
        };

        Ok(TransactionLoadData {
            fee: TransactionFee::default(), // This will be calculated later based on transaction type
            metadata,
        })
    }
}