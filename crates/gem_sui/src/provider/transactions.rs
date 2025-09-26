#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactions;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use crate::provider::transactions_mapper::{map_transaction, map_transaction_blocks};
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for SuiClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        let parts: Vec<&str> = data.split('_').collect();
        if parts.len() != 2 {
            return Err("Invalid transaction data format. Expected format: data_signature".into());
        }

        let transaction_data = parts[0].to_string();
        let signature = parts[1].to_string();

        Ok(self.broadcast(transaction_data, signature).await?.digest)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let transaction_blocks = self.get_checkpoint_transactions(block, None).await?;
        Ok(map_transaction_blocks(transaction_blocks))
    }

    async fn get_transactions_by_address(&self, address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self
            .get_transactions_by_address(address)
            .await?
            .data
            .into_iter()
            .flat_map(map_transaction)
            .collect())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactionState, ChainTransactions};
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = ChainTransactions::get_transactions_by_block(&client, latest_block - 1).await?;

        println!("Transactions in block {}: {}", latest_block - 1, transactions.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let request = TransactionStateRequest::new_id(TEST_TRANSACTION_ID.to_string());
        let status = client.get_transaction_status(request).await?;

        println!("Transaction status: {:?}", status);

        assert!(status.state == TransactionState::Confirmed);
        assert!(status.changes.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let transactions = ChainTransactions::get_transactions_by_address(&client, TEST_ADDRESS.to_string(), Some(1)).await?;
        println!("Address transactions count: {}", transactions.len());

        assert!(!transactions.is_empty());
        Ok(())
    }
}
