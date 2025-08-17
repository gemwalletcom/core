use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactions for CardanoClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.broadcast_transaction(data).await
    }

    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(TransactionUpdate {
            state: primitives::TransactionState::Confirmed,
            changes: vec![],
        })
    }
}
