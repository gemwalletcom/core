use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::{BroadcastOptions, Transaction, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::{provider::transactions_mapper::map_transactions, rpc::client::BitcoinClient};

#[async_trait]
impl<C: Client> ChainTransactions for BitcoinClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
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

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let mut transactions = Vec::new();
        let mut page = 1;

        loop {
            let block = self.get_block(block, page).await?;

            transactions.extend(map_transactions(self.get_chain(), block.txs));

            if block.total_pages == block.page {
                break;
            }

            page += 1;
        }

        Ok(transactions)
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let address_details = self.get_address_details(&address, limit.unwrap_or(25)).await?;
        let transactions = address_details.transactions.unwrap_or_default();
        Ok(map_transactions(self.get_chain(), transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
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

        let transactions = bitcoin_client.get_transactions_by_address(TEST_ADDRESS.to_string(), None).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());

        assert!(!transactions.is_empty());
    }
}
