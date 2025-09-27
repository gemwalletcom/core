use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionState;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::client::EthereumClient};
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionState for EthereumClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_receipt(&request.id).await?;
        Ok(map_transaction_status(&receipt))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client};
    use chain_traits::ChainTransactionState;
    use num_bigint::BigInt;
    use primitives::{TransactionChange, TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_ethereum_get_transaction_status_confirmed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let request = TransactionStateRequest::new_id("0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3".to_string());

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(42850974395536u64))]);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_transaction_status_confirmed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let request = TransactionStateRequest::new_id("0xd85c4496230adf8a7c0fc1e98713127fb31a0f8f72874acea443e2f615f3c1b6".to_string());

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(27753700000000u64))]);

        Ok(())
    }
}
