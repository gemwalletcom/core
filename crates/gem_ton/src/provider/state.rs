use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainState for TonClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_master_head().await?.first.root_hash)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_master_head().await?.last.seqno)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let chainhead = self.get_master_head().await?;
        state_mapper::map_node_status(&chainhead)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_ton_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let chain_id = client.get_chain_id().await?;
        println!("Ton chain ID: {}", chain_id);
        assert!(chain_id == "F6OpKZKqvqeFp6CQmFomXNMfMj2EnaUSOXN+Mh+wVWk=");
        Ok(())
    }

    #[tokio::test]
    async fn test_ton_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let latest_block = client.get_block_latest_number().await?;
        println!("Latest block: {}", latest_block);
        assert!(latest_block > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
