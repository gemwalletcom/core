use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};

use crate::provider::transactions_mapper::map_transaction;
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

    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number as i64).await?;
        let transactions = block
            .transactions
            .clone()
            .into_iter()
            .flat_map(|x| map_transaction(self.get_chain(), &block, &x))
            .collect::<Vec<Transaction>>();
        Ok(transactions)
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
