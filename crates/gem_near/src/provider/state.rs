use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::client::NearClient;

#[async_trait]
impl<C: Client + Clone> ChainState for NearClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_genesis_config().await?.chain_id)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_latest_block().await?.header.height)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let block = self.get_latest_block().await?;
        state_mapper::map_node_status(&block)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::rpc::client::NearClient;
    use chain_traits::ChainProvider;
    use gem_client::ReqwestClient;
    use gem_jsonrpc::{client::JsonRpcClient, new_client};

    #[tokio::test]
    async fn test_near_client_generic_interface() {
        let reqwest_client = ReqwestClient::new("https://example.com".to_string(), reqwest::Client::new());
        let jsonrpc_client = JsonRpcClient::new(reqwest_client);
        let near_client: NearClient<ReqwestClient> = NearClient::new(jsonrpc_client);

        assert_eq!(near_client.get_chain().to_string(), "near");
    }

    #[tokio::test]
    async fn test_near_genesis_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let jsonrpc_client = new_client("https://rpc.mainnet.near.org".to_string())?;
        let near_client: NearClient<gem_client::ReqwestClient> = NearClient::new(jsonrpc_client);

        let genesis_config = near_client.get_genesis_config().await?;
        assert_eq!(genesis_config.chain_id, "mainnet");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use chain_traits::ChainState;

        let jsonrpc_client = new_client("https://rpc.mainnet.near.org".to_string())?;
        let near_client: NearClient<gem_client::ReqwestClient> = NearClient::new(jsonrpc_client);
        let node_status = near_client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
