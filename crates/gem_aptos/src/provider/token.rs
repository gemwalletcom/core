use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use super::token_mapper::map_token_data;
use crate::models::CoinInfo;
use crate::rpc::client::AptosClient;

const FUNGIBLE_ASSET_METADATA_TYPE: &str = "0x1::fungible_asset::Metadata";
const MIN_TOKEN_ID_LENGTH: usize = 66;

enum TokenIdKind {
    CoinLegacy,
    FungibleAsset,
}

#[async_trait]
impl<C: Client> ChainToken for AptosClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let (address, resource_type) = match token_id_kind(&token_id) {
            Some(TokenIdKind::CoinLegacy) => {
                let parts: Vec<&str> = token_id.split("::").collect();
                if parts.len() < 3 {
                    return Err("Invalid token ID format".into());
                }
                (parts[0].to_string(), format!("0x1::coin::CoinInfo<{}>", token_id))
            }
            Some(TokenIdKind::FungibleAsset) => (token_id.clone(), FUNGIBLE_ASSET_METADATA_TYPE.to_string()),
            None => return Err("Invalid token ID format".into()),
        };

        let resource = self.get_account_resource::<CoinInfo>(address, &resource_type).await?;
        map_token_data(&resource, &token_id)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id_kind(token_id).is_some()
    }
}

fn token_id_kind(token_id: &str) -> Option<TokenIdKind> {
    if !token_id.starts_with("0x") || token_id.len() < MIN_TOKEN_ID_LENGTH {
        return None;
    }
    if token_id.contains("::") {
        Some(TokenIdKind::CoinLegacy)
    } else {
        Some(TokenIdKind::FungibleAsset)
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

    #[tokio::test]
    async fn test_aptos_get_token_data_legacy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let token_data = client
            .get_token_data("0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT".to_string())
            .await?;
        assert!(!token_data.name.is_empty());
        assert!(token_data.decimals > 0);
        println!("Legacy token data: {:?}", token_data);
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_fungible_asset_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let token_data = client
            .get_token_data("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b".to_string())
            .await?;
        assert_eq!(token_data.symbol, "USDt");
        assert_eq!(token_data.decimals, 6);
        println!("Fungible asset data: {:?}", token_data);
        Ok(())
    }
}
