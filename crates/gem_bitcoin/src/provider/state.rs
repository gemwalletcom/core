use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::Client;
use primitives::NodeSyncStatus;
use std::error::Error;

use crate::{provider::state_mapper, rpc::client::BitcoinClient};

#[async_trait]
impl<C: Client> ChainState for BitcoinClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let block = self.get_block_info(1).await?;
        block.previous_block_hash.ok_or_else(|| "Unable to get block hash".into())
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let node_info = self.get_node_info().await?;
        Ok(state_mapper::map_node_status(&node_info))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let node_info = self.get_node_info().await?;
        Ok(state_mapper::map_latest_block_number(&node_info))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_bitcoin_latest_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bitcoin_test_client();
        let block_number = client.get_block_latest_number().await?;

        assert!(block_number > 800_000, "Bitcoin block number should be above 800k, got: {}", block_number);
        println!("Bitcoin latest block: {}", block_number);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_bitcoin_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bitcoin_test_client();
        let chain_id = client.get_chain_id().await?;

        assert!(!chain_id.is_empty());
        assert!(chain_id.len() == 64); // Bitcoin block hashes are 64 characters
        println!("Bitcoin chain ID: {}", chain_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_bitcoin_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_bitcoin_test_client();
        let status = client.get_node_status().await?;

        println!("Bitcoin node status: {:#?}", status);

        assert!(status.in_sync);
        assert!(status.latest_block_number.unwrap_or(0) > 0);
        assert!(status.current_block_number.unwrap_or(0) > 0);
        assert!(status.latest_block_number == status.current_block_number);

        Ok(())
    }
}
