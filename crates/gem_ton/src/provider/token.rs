use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainToken for TonClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        self.get_token_data(token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("EQ") && token_id.len() >= 40 && token_id.len() <= 60
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;

    #[tokio::test]
    async fn test_ton_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ton_test_client();
        let token_data = client.get_token_data("EQACLXDwit01stiqK9FvYiJo15luVzfD5zU8uwDSq6JXxbP8".to_string()).await?;
        println!("Token data: {:?}", token_data);

        assert!(token_data.name == "Spintria");
        assert!(token_data.symbol == "SP");
        assert!(token_data.decimals == 8);

        Ok(())
    }
}
