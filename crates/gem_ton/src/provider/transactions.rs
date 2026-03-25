use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{provider::transactions_mapper::map_transactions, rpc::client::TonClient};

#[async_trait]
impl<C: Client> ChainTransactions for TonClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transactions_by_masterchain_block(block.to_string()).await?;
        Ok(map_transactions(transactions.transactions))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_transaction(hash).await?.transactions).into_iter().next())
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(100);
        let transactions = self.get_transactions_by_address(address, limit).await?;
        Ok(map_transactions(transactions.transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_ton_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let latest_block = ChainState::get_block_latest_number(&create_ton_test_client()).await?;
        let transactions = ChainTransactions::get_transactions_by_block(&create_ton_test_client(), latest_block).await?;

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());

        assert!(!transactions.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transactions = ChainTransactions::get_transactions_by_address(&create_ton_test_client(), TransactionsRequest::new(TEST_ADDRESS.to_string()).with_limit(10)).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());

        assert!(!transactions.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let transaction = ChainTransactions::get_transaction_by_hash(&client, TEST_TRANSACTION_ID.to_string()).await?.unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
