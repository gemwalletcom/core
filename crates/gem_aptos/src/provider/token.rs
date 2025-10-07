use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use super::token_mapper::map_token_data;
use crate::models::CoinInfo;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainToken for AptosClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let parts: Vec<&str> = token_id.split("::").collect();
        if parts.len() < 2 {
            return Err("Invalid token ID format".into());
        }

        let address = parts[0];
        let resource_type = format!("0x1::coin::CoinInfo<{}>", token_id);

        let resource = self.get_account_resource::<CoinInfo>(address.to_string(), &resource_type).await?;
        map_token_data(&resource, &token_id)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("0x") && token_id.len() >= 66 && token_id.contains("::")
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::create_aptos_test_client;
    use chain_traits::ChainToken;

    #[tokio::test]
    async fn test_aptos_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let token_data = client
            .get_token_data("0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC".to_string())
            .await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        println!("Token data: {:?}", token_data);
        Ok(())
    }
}
