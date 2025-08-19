use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.send_transaction(data, None).await
    }
    
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;
        
        if transaction.slot > 0 {
            if transaction.meta.err.is_some() {
                Ok(TransactionUpdate {
                    state: TransactionState::Failed,
                    changes: vec![],
                })
            } else {
                Ok(TransactionUpdate {
                    state: TransactionState::Confirmed,
                    changes: vec![],
                })
            }
        } else {
            Ok(TransactionUpdate {
                state: TransactionState::Pending,
                changes: vec![],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::SolanaTransaction;
    use primitives::JsonRpcResult;

    #[test]
    fn test_get_transaction_status() {
        let result: JsonRpcResult<SolanaTransaction> = serde_json::from_str(include_str!("../../testdata/transaction_state_transfer_sol.json")).unwrap();
        let transaction = result.result;
        
        let state = if transaction.slot > 0 {
            if transaction.meta.err.is_some() {
                TransactionState::Failed
            } else {
                TransactionState::Confirmed
            }
        } else {
            TransactionState::Pending
        };

        assert_eq!(state, TransactionState::Confirmed);
        assert_eq!(transaction.slot, 361169359);
    }
}