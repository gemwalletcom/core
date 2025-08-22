use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainTransactions for TronClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;
        map_transaction_broadcast(&response)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_reciept(request.id).await?;
        Ok(map_transaction_status(&receipt))
    }
}
