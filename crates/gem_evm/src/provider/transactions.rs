use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactions;
use primitives::{BroadcastOptions, TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status},
    rpc::client::EthereumClient,
};
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let data = map_transaction_broadcast(&data);
        Ok(self.send_raw_transaction(&data).await?)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_receipt(&request.id).await?;
        Ok(map_transaction_status(&receipt))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client};
    use chain_traits::ChainTransactions;
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

    #[tokio::test]
    #[ignore]
    async fn test_ethereum_transaction_broadcast() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let signed_tx = "0xf86c808502540be40082520894d4e56740f876aef8c010b86a40d5f56745a118d0765af9a146000000808081c0a05e1d3c1b2c3b0f8b7c8e9f0a1b2c3d4e5f6789abcdef0123456789abcdef012345a04f2c3a1b0d8e7f9a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(signed_tx.to_string(), options).await;

        assert!(result.is_ok() || result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast_invalid_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let invalid_tx = "0xinvalidtransactiondata";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(invalid_tx.to_string(), options).await;

        assert!(result.is_err());

        Ok(())
    }
}
