use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use super::transactions_mapper::{map_transaction_broadcast, map_transaction_status};
use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result).map_err(|e| e.into())
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&transaction))
    }
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
mod integration_tests {
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
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        Ok(())
    }
}
