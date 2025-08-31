use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client> ChainToken for TronClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Self::get_token_data(self, token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("T") && token_id.len() >= 30
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_test_client;

    const TEST_USDT_TOKEN_ID: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

    #[tokio::test]
    async fn test_get_token_data() {
        let tron_client = create_test_client();

        let asset = tron_client.get_token_data(TEST_USDT_TOKEN_ID.to_string()).await.unwrap();

        assert_eq!(asset.symbol, "USDT");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.id.token_id, Some(TEST_USDT_TOKEN_ID.to_string()));
        println!("Token data: {}", asset.name);
    }

    #[tokio::test]
    async fn test_get_is_token_address() {
        let tron_client = create_test_client();

        assert!(tron_client.get_is_token_address(TEST_USDT_TOKEN_ID));
        assert!(!tron_client.get_is_token_address("TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH"));
        assert!(!tron_client.get_is_token_address("invalid"));
    }
}
