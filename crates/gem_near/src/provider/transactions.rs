use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.broadcast_transaction(&data).await?.transaction.hash)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let result = self.get_transaction_status(&request.id, &request.sender_address).await?;

        let state = match result.final_execution_status.as_str() {
            "FINAL" => TransactionState::Confirmed,
            _ => TransactionState::Pending,
        };

        Ok(TransactionUpdate { state, changes: vec![] })
    }
}
