use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    provider::transactions_mapper::{map_transaction, map_transactions},
    rpc::client::AptosClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for AptosClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_block_transactions(block).await?.transactions))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transaction(self.get_transaction_by_hash(&hash).await?))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, .. } = request;
        Ok(map_transactions(self.get_transactions_by_address(address).await?))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_aptos_test_client};
    use chain_traits::{ChainState, ChainTransactions};

    #[tokio::test]
    async fn test_aptos_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let _latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(100000).await?;
        println!("Transactions in block 100000: {}", transactions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        Ok(())
    }
}
