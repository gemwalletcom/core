use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::client::TonClient};

#[async_trait]
impl<C: Client> ChainTransactionState for TonClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transaction(request.id.clone()).await?;
        map_transaction_status(request, transactions)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainTransactionState;
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_ton_transaction_status_confirmed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let request = TransactionStateRequest::new_id("w7Tz84LDLoQ3HPCuU0DZbj2sCq-eZKH1vse_wm67kxA=".to_string());
        let update = client.get_transaction_status(request).await?;

        assert_eq!(update.state, TransactionState::Confirmed);

        println!("Transaction update: {:?}", update);

        Ok(())
    }

    #[tokio::test]
    async fn test_ton_transaction_status_reverted() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let request = TransactionStateRequest::new_id("PF2o0flrhohsrneG5rCeMnX8JdmZo0Ytsr_P0aOZWA8=".to_string());
        let update = client.get_transaction_status(request).await?;
        assert_eq!(update.state, TransactionState::Failed);

        println!("Transaction update: {:?}", update);

        Ok(())
    }
}
