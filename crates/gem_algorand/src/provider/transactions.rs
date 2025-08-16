use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate, TransactionState, TransactionChange};

use crate::rpc::client::AlgorandClient;
use super::transactions_mapper::map_transaction_broadcast;

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result).map_err(|e| e.into())
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction_status(&request.id).await?;
        
        let state: TransactionState = if transaction.confirmed_round > 0 {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        
        let mut changes = Vec::new();
        if transaction.confirmed_round > 0 {
            changes.push(TransactionChange::BlockNumber(transaction.confirmed_round.to_string()));
        }
        
        Ok(TransactionUpdate {
            state,
            changes,
        })
    }
}