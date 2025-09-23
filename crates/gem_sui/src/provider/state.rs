use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainState;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainState for SuiClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_chain_id().await
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_latest_block().await
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let latest_checkpoint = self.get_block_latest_number().await?;
        state_mapper::map_node_status(latest_checkpoint)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::*;

    #[tokio::test]
    async fn test_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let chain_id = client.get_chain_id().await?;
        assert!(!chain_id.is_empty());
        println!("Sui chain ID: {}", chain_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        assert!(latest_block > 0);
        println!("Latest block: {}", latest_block);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
