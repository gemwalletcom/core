use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use crate::rpc::client::XRPClient;
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainState for XRPClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger_current().await?.ledger_current_index as u64)
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
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
}
