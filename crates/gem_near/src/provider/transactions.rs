use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transactions_mapper::map_transaction_status, rpc::client::NearClient};

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.broadcast_transaction(&data).await?.transaction.hash)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let result = self.get_transaction_status(&request.id, &request.sender_address).await?;
        Ok(map_transaction_status(&result))
    }
}
