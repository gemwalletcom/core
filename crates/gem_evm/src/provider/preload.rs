use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use primitives::{TransactionLoadMetadata, TransactionPreloadInput};

use crate::provider::preload_mapper::map_transaction_preload;
use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for EthereumClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let nonce = self.get_transaction_count(&input.sender_address).await?;
        let chain_id = self.chain.to_chain().network_id().to_string();

        map_transaction_preload(nonce, chain_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client, TEST_ADDRESS};
    use primitives::{Asset, Chain, TransactionInputType};

    #[tokio::test]
    async fn test_ethereum_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        println!("metadata: {:#?}", metadata);

        assert!(metadata.get_sequence()? > 0);
        assert_eq!(metadata.get_chain_id()?, "1");

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::SmartChain)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        println!("metadata: {:#?}", metadata);

        assert!(metadata.get_sequence()? > 0);
        assert_eq!(metadata.get_chain_id()?, "56");

        Ok(())
    }
}
