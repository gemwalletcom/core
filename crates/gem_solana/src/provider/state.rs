use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainState for SolanaClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_genesis_hash().await
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_slot().await
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let slot = self.get_slot().await?;
        state_mapper::map_node_status(slot)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_solana_test_client;

    #[tokio::test]
    async fn test_solana_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let chain_id = client.get_chain_id().await?;

        println!("Solana chain ID: {}", chain_id);

        assert!(chain_id == "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d");
        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let latest_block = client.get_block_latest_number().await?;

        assert!(latest_block > 0);
        println!("Latest block number: {}", latest_block);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
