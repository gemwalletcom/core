use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainTransactions for TronClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;
        map_transaction_broadcast(&response)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_reciept(request.id).await?;
        Ok(map_transaction_status(&receipt))
    }

    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let tron_client = create_test_client();

        let latest_block = tron_client.get_block_latest_number().await.unwrap();
        let transactions = tron_client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let tron_client = create_test_client();

        let transactions = tron_client.get_transactions_by_address(TEST_ADDRESS.to_string()).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}
