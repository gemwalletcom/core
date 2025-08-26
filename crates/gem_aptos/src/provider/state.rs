use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainState for AptosClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.chain_id.to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger().await?.block_height)
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
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
}
