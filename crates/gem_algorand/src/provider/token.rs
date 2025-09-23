use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::{
    AlgorandClientIndexer,
    provider::token_mapper::{is_valid_token_id, map_asset},
    rpc::client::AlgorandClient,
};

#[async_trait]
impl<C: Client> ChainToken for AlgorandClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let asset = self.get_asset(&token_id).await?;
        Ok(map_asset(asset))
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        is_valid_token_id(token_id)
    }
}

#[async_trait]
impl<C: Client> ChainToken for AlgorandClientIndexer<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let asset = self.get_asset(&token_id).await?.asset;
        Ok(map_asset(asset))
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        is_valid_token_id(token_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainToken;

    #[tokio::test]
    async fn test_algorand_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let token_data = client.get_token_data("31566704".to_string()).await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        println!("Token data: {:?}", token_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_token_data_indexer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_indexer_client();
        let token_data = client.get_token_data("31566704".to_string()).await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        println!("Token data: {:?}", token_data);
        Ok(())
    }
}
