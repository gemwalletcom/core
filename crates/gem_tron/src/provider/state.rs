use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainState for TronClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_latest_block().await? as u64)
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
}
