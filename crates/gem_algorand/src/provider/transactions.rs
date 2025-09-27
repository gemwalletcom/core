use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use crate::{
    provider::transactions_mapper::{map_transaction_broadcast, map_transactions},
    rpc::client::AlgorandClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result).map_err(|e| e.into())
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.indexer.get_block(block).await?;
        Ok(map_transactions(block.transactions))
    }

    async fn get_transactions_by_address(&self, address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.indexer.get_account_transactions(&address).await?;
        Ok(map_transactions(transactions.transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactions};

    #[tokio::test]
    async fn test_algorand_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(latest_block - 1).await?;
        println!("Transactions in block {}: {}", latest_block - 1, transactions.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string(), None).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());

        assert!(!transactions.is_empty());
        Ok(())
    }
}
