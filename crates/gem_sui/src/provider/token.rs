use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainToken;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::Asset;

use crate::provider::token_mapper::{map_is_token_address, map_token_data};
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainToken for SuiClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let metadata = self.get_coin_metadata(token_id.clone()).await?;
        Ok(map_token_data(self.chain, &token_id, metadata))
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        map_is_token_address(token_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainToken;

    #[tokio::test]
    async fn test_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let token_data = client.get_token_data(TEST_TOKEN_ADDRESS.to_string()).await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        println!("Token data: {:?}", token_data);
        Ok(())
    }
}

