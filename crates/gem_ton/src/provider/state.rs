use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainState for TonClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_master_head().await?.result.initial.root_hash)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_latest_block().await? as u64)
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
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
}
