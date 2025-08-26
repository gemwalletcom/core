use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::AlgorandClient;

#[async_trait]
impl<C: Client> ChainState for AlgorandClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        // Algorand mainnet genesis ID is always this value
        Ok("mainnet-v1.0".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let block_headers = self.get_block_headers().await?;
        Ok(block_headers.current_round as u64)
    }
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
mod integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_algorand_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let chain_id = client.get_chain_id().await?;
        assert!(!chain_id.is_empty());
        println!("Algorand chain ID: {}", chain_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let latest_block = client.get_block_latest_number().await?;
        assert!(latest_block > 0);
        println!("Latest block: {}", latest_block);
        Ok(())
    }
}
