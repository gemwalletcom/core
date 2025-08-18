use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput, TransactionLoadInput, TransactionLoadData};

use crate::{rpc::client::CosmosClient, provider::preload_mapper};

#[async_trait]
impl<C: Client> ChainPreload for CosmosClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        Ok(TransactionPreload {
            chain_id: self.get_chain().as_chain().network_id().to_string(),
            ..TransactionPreload::default()
        })
    }
    
    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&input.sender_address).await?;
        let fee = preload_mapper::calculate_transaction_fee(&input.input_type, self.get_chain(), &input.gas_price);
        
        Ok(TransactionLoadData::builder()
            .account_number(account.account_number.parse().unwrap_or(0))
            .sequence(account.sequence.parse().unwrap_or(0))
            .fee(fee)
            .build())
    }
}