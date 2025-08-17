use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::NearClient;
use super::preload_mapper;

#[async_trait]
impl<C: Client> ChainPreload for NearClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let public_key = preload_mapper::address_to_public_key(&input.sender_address)?;
        let access_key = self.get_near_account_access_key(&input.sender_address, &public_key).await?;
        let block = self.get_near_latest_block().await?;
        let is_destination_address_exist = self.get_near_account(&input.destination_address).await.is_ok();
        
        Ok(preload_mapper::map_transaction_preload(&input, &access_key, &block, is_destination_address_exist))
    }
}