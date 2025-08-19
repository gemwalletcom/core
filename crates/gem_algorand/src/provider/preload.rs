use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainPreload for AlgorandClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let params = self.get_transactions_params().await?;

        Ok(TransactionPreload {
            block_hash: params.genesis_hash,
            block_number: params.last_round,
            utxos: vec![],
            sequence: params.last_round as u64,
            chain_id: params.genesis_id,
            is_destination_address_exist: true,
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: TransactionLoadMetadata::Algorand { 
                sequence: input.sequence 
            },
        })
    }
}
