use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use crate::provider::state_mapper;
use crate::rpc::client::XRPClient;
use gem_client::Client;
use primitives::NodeSyncStatus;

#[async_trait]
impl<C: Client> ChainState for XRPClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger_current().await?.ledger_current_index as u64)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let ledger_info = self.get_ledger_current().await?;
        state_mapper::map_node_status(&ledger_info)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_xrp_test_client;

    #[tokio::test]
    async fn test_get_xrp_latest_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let block_number = client.get_block_latest_number().await?;

        assert!(block_number > 80_000_000, "XRP ledger index should be above 80M, got: {}", block_number);
        println!("XRP latest ledger: {}", block_number);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 80_000_000);

        Ok(())
    }
}
