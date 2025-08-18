use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::AptosClient;
use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};

#[async_trait]
impl<C: Client> ChainTransactions for AptosClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.submit_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction_by_hash(&request.id).await?;
        Ok(map_transaction_status(&transaction))
    }
}