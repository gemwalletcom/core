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

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactions};
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_bitcoin_get_transactions_status() {
        let bitcoin_client = create_bitcoin_test_client();

        let request = TransactionStateRequest::new_id(TEST_TRANSACTION_ID.to_string());
        let update = bitcoin_client.get_transaction_status(request).await.unwrap();

        println!("State: {}", update.state);
        assert!(update.state == TransactionState::Confirmed);
    }

    #[tokio::test]
    async fn test_bitcoin_get_transactions_by_block() {
        let bitcoin_client = create_bitcoin_test_client();

        let latest_block = bitcoin_client.get_block_latest_number().await.unwrap();
        let transactions = bitcoin_client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
    }

    #[tokio::test]
    async fn test_bitcoin_get_transactions_by_address() {
        let bitcoin_client = create_bitcoin_test_client();

        let transactions = bitcoin_client.get_transactions_by_address(TEST_ADDRESS.to_string()).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}
