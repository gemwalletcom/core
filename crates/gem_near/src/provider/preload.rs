use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};
use primitives::transaction_load::TransactionLoadMetadata;

use super::preload_mapper;
use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainPreload for NearClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let public_key = preload_mapper::address_to_public_key(&input.sender_address)?;
        let access_key = self.get_account_access_key(&input.sender_address, &public_key).await?;
        let block = self.get_latest_block().await?;
        let is_destination_address_exist = self.get_account(&input.destination_address).await.is_ok();

        Ok(preload_mapper::map_transaction_preload(
            &input,
            &access_key,
            &block,
            is_destination_address_exist,
        ))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: TransactionLoadMetadata::Near {
                sequence: input.sequence,
                block_hash: input.block_hash,
                is_destination_exist: true, // Use a default or extract from input if needed
            },
        })
    }
}
