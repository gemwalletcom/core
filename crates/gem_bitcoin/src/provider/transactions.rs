use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::{Transaction, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainTransactions for BitcoinClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(data).await?;

        if let Some(error) = result.error {
            return Err(error.message.into());
        }

        result.result.ok_or_else(|| "unknown hash".into())
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;
        let status = if transaction.block_height > 0 {
            TransactionState::Confirmed
        } else {
            TransactionState::Pending
        };
        Ok(TransactionUpdate::new_state(status))
    }

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
