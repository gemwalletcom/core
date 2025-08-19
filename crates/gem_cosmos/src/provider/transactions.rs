use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::map_transaction_broadcast;
use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainTransactions for CosmosClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&response)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(request.id).await?;

        let state = if transaction.tx_response.code == 0 {
            TransactionState::Confirmed
        } else {
            TransactionState::Reverted
        };

        Ok(TransactionUpdate::new_state(state))
    }
}
