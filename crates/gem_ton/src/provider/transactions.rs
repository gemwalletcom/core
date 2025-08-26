use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};

use crate::provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status, map_transactions};
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainTransactions for TonClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(data).await?.result;
        map_transaction_broadcast(result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transaction(request.id.clone()).await?;
        map_transaction_status(request, transactions)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transactions_by_masterchain_block(block.to_string()).await?;
        Ok(map_transactions(transactions.transactions))
    }

    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transactions_by_address(address, 100).await?;
        Ok(map_transactions(transactions.transactions))
    }
}

// #[cfg(all(test, feature = "integration_tests"))]
// mod integration_tests {
//     use super::*;
//     use crate::provider::testkit::{create_ton_test_client, TEST_ADDRESS};
//     use chain_traits::ChainState;

//     #[tokio::test]
//     async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         let client = create_ton_test_client();
//         let latest_block = client.get_block_latest_number().await?;
//         let transactions = client.get_transactions_by_block(latest_block).await?;
//         println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         let client = create_ton_test_client();
//         let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string(), 10).await?;
//         println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.transactions.len());
//         Ok(())
//     }
// }
