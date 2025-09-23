use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainState for AptosClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.chain_id.to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.block_height)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let ledger = self.get_ledger().await?;
        state_mapper::map_node_status(&ledger)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::create_aptos_test_client;
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_aptos_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let chain_id = client.get_chain_id().await?;
        assert!(!chain_id.is_empty());
        println!("Aptos chain ID: {}", chain_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let latest_block = client.get_block_latest_number().await?;
        assert!(latest_block > 0);
        println!("Latest block: {}", latest_block);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
