use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use super::transactions_mapper::{map_transaction_broadcast, map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for TronClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;
        map_transaction_broadcast(&response)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block_tranactions(block).await?;
        if block_data.transactions.is_empty() {
            return Ok(vec![]);
        }

        let receipts = self.get_block_tranactions_reciepts(block).await?;
        Ok(map_transactions_by_block(self.get_chain(), block_data, receipts))
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
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
    use crate::provider::testkit::{TEST_ADDRESS, create_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let tron_client = create_test_client();

        let latest_block = tron_client.get_block_latest_number().await.unwrap();
        let block_number = latest_block - 25;
        let transactions = tron_client.get_transactions_by_block(block_number).await.unwrap();

        println!(
            "Latest block: {}, test block: {}, transactions count: {}",
            latest_block,
            block_number,
            transactions.len()
        );
        assert!(latest_block > 0);
        assert!(!transactions.is_empty());

        if let Some(transaction) = transactions.first() {
            println!("First transaction ID: {}", transaction.id);
            assert!(!transaction.id.is_empty());
        }
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let tron_client = create_test_client();
        let transactions = tron_client.get_transactions_by_address(TEST_ADDRESS.to_string(), Some(1)).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        assert!(!transactions.is_empty());
    }
}
