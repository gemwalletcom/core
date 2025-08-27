use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status, map_transactions},
    rpc::client::StellarClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for StellarClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let result = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&result))
    }

    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let payments = self.get_account_payments(address).await?;
        Ok(map_transactions(self.get_chain(), payments))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let payments = self.get_block_payments_all(block as i64).await?;
        Ok(map_transactions(self.get_chain(), payments))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let stellar_client = create_test_client();
        let latest_block = stellar_client.get_block_latest_number().await.unwrap();
        let transactions = stellar_client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let stellar_client = create_test_client();
        let transactions = stellar_client.get_transactions_by_address(TEST_ADDRESS.to_string()).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}
