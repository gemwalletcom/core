use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use super::transactions_mapper::{map_transaction, map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for TronClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block_tranactions(block).await?;
        if block_data.transactions.is_empty() {
            return Ok(vec![]);
        }

        let receipts = self.get_block_tranactions_reciepts(block).await?;
        Ok(map_transactions_by_block(self.get_chain(), block_data, receipts))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction(
            self.get_chain(),
            self.get_transaction(hash.clone()).await?,
            self.get_transaction_reciept(hash).await?,
        ))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(20);
        let transactions = self.trongrid_client.get_transactions_by_address(&address, limit).await?.data;

        if transactions.is_empty() {
            return Ok(vec![]);
        }

        let futures = transactions.iter().map(|tx| self.get_transaction_reciept(tx.tx_id.clone()));
        let receipts = futures::future::try_join_all(futures).await?;

        Ok(map_transactions_by_address(transactions, receipts))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let tron_client = create_test_client();

        let latest_block = tron_client.get_block_latest_number().await.unwrap();
        let block_number = latest_block - 25;
        let transactions = tron_client.get_transactions_by_block(block_number).await.unwrap();

        println!("Latest block: {}, test block: {}, transactions count: {}", latest_block, block_number, transactions.len());
        assert!(latest_block > 0);
        assert!(!transactions.is_empty());

        if let Some(transaction) = transactions.first() {
            println!("First transaction ID: {}", transaction.id.hash);
            assert!(!transaction.id.hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let tron_client = create_test_client();
        let transactions = tron_client
            .get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string()).with_limit(1))
            .await
            .unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let tron_client = create_test_client();
        let transaction = tron_client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await.unwrap().unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
    }
}
