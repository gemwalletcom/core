use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use super::token_mapper::map_token_data;
use crate::models::CoinInfo;
use crate::rpc::client::AptosClient;
use crate::token_id::is_fungible_asset_token_id;

const FUNGIBLE_ASSET_METADATA_TYPE: &str = "0x1::fungible_asset::Metadata";

#[async_trait]
impl<C: Client> ChainToken for AptosClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let resource = self.get_account_resource::<CoinInfo>(token_id.clone(), FUNGIBLE_ASSET_METADATA_TYPE).await?;
        map_token_data(&resource, &token_id)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        is_fungible_asset_token_id(token_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::create_aptos_test_client;
    use chain_traits::ChainToken;
    use primitives::asset_constants::APTOS_USDT_TOKEN_ID;

    #[tokio::test]
    async fn test_aptos_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let token_data = client.get_token_data(APTOS_USDT_TOKEN_ID.to_string()).await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        assert_eq!(token_data.symbol, "USDt");
        assert_eq!(token_data.decimals, 6);
        Ok(())
    }
}
