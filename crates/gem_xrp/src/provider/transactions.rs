use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::provider::transactions_mapper::{map_direct_transaction, map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for XRPClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let ledger = self.get_block_transactions(block).await?;
        Ok(map_transactions_by_block(ledger))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(100);
        let account_ledger = self.get_account_transactions(address, limit).await?;
        Ok(map_transactions_by_address(account_ledger))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&hash).await?;
        Ok(map_direct_transaction(self.get_chain(), transaction))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_TRANSACTION_ID, create_xrp_test_client};

    #[tokio::test]
    async fn test_xrp_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let transaction = client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await?.unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
