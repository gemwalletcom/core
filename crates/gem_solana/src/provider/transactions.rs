use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transaction_mapper::{map_block_transactions, map_signatures_transactions},
    rpc::client::SolanaClient,
};

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

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_transactions = self.get_block_transactions(block).await?;
        Ok(map_block_transactions(&block_transactions))
    }

    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let signatures = self.get_signatures_for_address(&address, 10).await?;
        if signatures.is_empty() {
            return Ok(vec![]);
        }
        let signatures_ids = signatures.clone().iter().map(|x| x.signature.clone()).collect();
        let transactions = self.get_transactions(signatures_ids).await?;
        Ok(map_signatures_transactions(transactions, signatures))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let client = create_test_client();

        let latest_block = client.get_block_latest_number().await.unwrap();
        let transactions = client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let client = create_test_client();

        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}
