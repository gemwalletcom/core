#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactions, TransactionsRequest};
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::Transaction;

use crate::provider::transactions_mapper::{map_transaction, map_transaction_blocks};
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for SuiClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let transaction_blocks = self.get_checkpoint_transactions(block, None).await?;
        Ok(map_transaction_blocks(transaction_blocks))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(map_transaction(self.get_transaction(hash).await?))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let TransactionsRequest { address, .. } = request;
        Ok(self.get_transactions_by_address(address).await?.data.into_iter().flat_map(map_transaction).collect())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactionState, ChainTransactions, TransactionsRequest};
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
        let transactions = ChainTransactions::get_transactions_by_address(&client, TransactionsRequest::new(TEST_ADDRESS.to_string()).with_limit(1)).await?;
        println!("Address transactions count: {}", transactions.len());

        assert!(!transactions.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let transaction = ChainTransactions::get_transaction_by_hash(&client, TEST_TRANSACTION_ID.to_string()).await?.unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
