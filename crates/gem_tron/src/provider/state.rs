use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainState for TronClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_latest_block().await? as u64)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Send + Sync>> {
        let latest_block = self.get_block_latest_number().await? as i64;
        state_mapper::map_node_status(latest_block)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_test_client;

    #[tokio::test]
    async fn test_get_chain_id() {
        let tron_client = create_test_client();

        let chain_id = tron_client.get_chain_id().await.unwrap();

        // Tron doesn't have a traditional chain ID like Ethereum
        assert_eq!(chain_id, "");
    }

    #[tokio::test]
    async fn test_get_block_latest_number() {
        let tron_client = create_test_client();

        let latest_block = tron_client.get_block_latest_number().await.unwrap();

        // Latest block should be a positive number
        assert!(latest_block > 0);
        println!("Latest block number: {}", latest_block);
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tron_client = create_test_client();
        let node_status = tron_client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
