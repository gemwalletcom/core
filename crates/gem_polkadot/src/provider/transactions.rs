use async_trait::async_trait;
use chain_traits::{ChainProvider, ChainTransactions};
use std::error::Error;

use gem_client::Client;
use primitives::{BroadcastOptions, Transaction};

use super::transactions_mapper;
use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainTransactions for PolkadotClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.broadcast_transaction(data).await?;

        if let Some(hash) = response.hash {
            Ok(hash)
        } else if let Some(error) = response.error {
            let cause = response.cause.unwrap_or_default();
            Err(format!("{}: {}", error, cause).into())
        } else {
            Err("Invalid broadcast response".into())
        }
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block(block as i64).await?;
        Ok(transactions_mapper::map_transactions(self.get_chain(), block_data))
    }

    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_polkadot_test_client;
    use chain_traits::ChainTransactionState;
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_polkadot_get_transaction_status_failed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let request =
            TransactionStateRequest::new_id("0x3a9dda661cbdfe12e15c623cd14abf3da64d4bcbe11c0c776def748713c2248b".to_string()).with_block_number(27_830_222);

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Failed);
        assert!(result.changes.is_empty());

        Ok(())
    }
}
