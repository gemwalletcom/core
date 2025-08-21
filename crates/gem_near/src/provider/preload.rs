use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use primitives::{FeeRate, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use super::{preload_mapper, state_mapper};
use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for NearClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
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
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        state_mapper::map_gas_price_to_priorities(&gas_price)
    }
}
