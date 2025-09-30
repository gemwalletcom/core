use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainToken for HyperCoreClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let spot_metadata = self.get_spot_metadata().await?;
        let token = spot_metadata
            .0
            .tokens
            .iter()
            .find(|t| t.name == token_id)
            .ok_or(format!("Token not found with symbol: {}", token_id))?;

        let asset_id = AssetId::from(
            self.chain,
            Some(AssetId::sub_token_id(&[token_id.clone(), token.token_id.clone(), token.index.to_string()])),
        );

        Ok(Asset {
            id: asset_id.clone(),
            chain: self.chain,
            token_id: asset_id.token_id.clone(),
            name: token.name.clone(),
            symbol: token.name.clone(),
            decimals: token.wei_decimals,
            asset_type: AssetType::TOKEN,
        })
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod tests {
    use super::*;
    use crate::provider::testkit::{USDC_TOKEN_ID, create_hypercore_test_client};

    #[tokio::test]
    #[ignore]
    async fn test_get_token_data_usdc() {
        let client = create_hypercore_test_client();

        let asset = client.get_token_data("USDC".to_string()).await.unwrap();

        assert_eq!(asset.symbol, "USDC");
        assert_eq!(asset.decimals, 8);
        assert_eq!(asset.chain, primitives::Chain::HyperCore);
        assert_eq!(asset.asset_type, AssetType::TOKEN);
        assert_eq!(asset.token_id, Some(USDC_TOKEN_ID.to_string()));
    }
}
