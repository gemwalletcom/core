use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::provider::transactions_mapper::map_transaction_broadcast;
use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client> ChainTransactions for XRPClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let status = self.get_transaction_status(&request.id).await?;

        let transaction_state = if status.status == "success" {
            primitives::TransactionState::Confirmed
        } else {
            primitives::TransactionState::Pending
        };

        Ok(TransactionUpdate::new_state(transaction_state))
    }
}
