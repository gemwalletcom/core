use async_trait::async_trait;
use chain_traits::ChainTransactions;
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction, TransactionStateRequest, TransactionUpdate};

use crate::{
    models::AccountResult,
    provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status, map_transactions},
    rpc::client::StellarClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for StellarClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let result = self.broadcast_transaction(&data).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let result = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&result))
    }

    async fn get_transactions_by_address(&self, address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let payments = self.get_account_payments(address).await?;
        match payments {
            AccountResult::Found(payments) => Ok(map_transactions(self.get_chain(), payments._embedded.records)),
            AccountResult::NotFound => Ok(Vec::new()),
        }
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let payments = self.get_block_payments_all(block).await?;
        Ok(map_transactions(self.get_chain(), payments))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EMPTY_ADDRESS, create_test_client};
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
        let transactions = stellar_client.get_transactions_by_address(TEST_ADDRESS.to_string(), None).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());

        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_by_address_empty() {
        let stellar_client = create_test_client();
        let transactions = stellar_client.get_transactions_by_address(TEST_EMPTY_ADDRESS.to_string(), None).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_EMPTY_ADDRESS, transactions.len());

        assert!(transactions.is_empty());
    }
}
