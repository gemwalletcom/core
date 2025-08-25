use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};

use crate::provider::transactions_mapper;
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainTransactions for TonClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(data).await?.result;
        transactions_mapper::map_transaction_broadcast(result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transaction(request.id.clone()).await?;
        transactions_mapper::map_transaction_status(request, transactions)
    }

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
