use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainState for PolkadotClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_version().await?.chain)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_block_header("head").await?.number)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_polkadot_test_client;

    #[tokio::test]
    async fn test_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let chain_id = client.get_chain_id().await?;
        assert!(!chain_id.is_empty());
        println!("Chain ID: {}", chain_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let block_number = client.get_block_latest_number().await?;
        assert!(block_number > 0);
        println!("Latest block: {}", block_number);
        Ok(())
    }
}
