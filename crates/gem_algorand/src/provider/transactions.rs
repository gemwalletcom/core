use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};
use crate::{provider::transactions_mapper::map_transactions, rpc::client::AlgorandClient, AlgorandClientIndexer};

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result).map_err(|e| e.into())
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&transaction))
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandClientIndexer<C> {
    async fn transaction_broadcast(&self, _data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_block(block).await?;
        Ok(map_transactions(transactions.transactions))
    }

    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_account_transactions(&address).await?;
        Ok(map_transactions(transactions.transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactions};

    #[tokio::test]
    async fn test_algorand_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_indexer_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(latest_block - 1).await?;
        println!("Transactions in block {}: {}", latest_block - 1, transactions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_indexer_client();
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        Ok(())
    }
}
