use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionState, TransactionStateRequest, TransactionUpdate};

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

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let solana_client = create_test_client();

        let latest_block = solana_client.get_block_latest_number().await.unwrap();
        let transactions = solana_client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let solana_client = create_test_client();

        let transactions = solana_client.get_transactions_by_address(TEST_ADDRESS.to_string()).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::SolanaTransaction;
    use gem_jsonrpc::types::JsonRpcErrorResponse;
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

    #[test]
    fn test_transaction_broadcast_error() {
        let error_response: JsonRpcErrorResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_swap_error.json")).unwrap();

        assert_eq!(error_response.error.code, -32002);
        assert_eq!(
            error_response.error.message,
            "Transaction simulation failed: Error processing Instruction 3: custom program error: 0x1771"
        );
        assert_eq!(error_response.id, 1755839259);
    }
}
