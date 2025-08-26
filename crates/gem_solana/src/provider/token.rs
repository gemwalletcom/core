use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::{
    model::Extension,
    rpc::{client::SolanaClient, mapper::SolanaMapper},
};

#[async_trait]
impl<C: Client + Clone> ChainToken for SolanaClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let token_info_result = self.get_token_mint_info(&token_id).await?;
        let token_info = token_info_result.info();

        if let Some(extensions) = &token_info.extensions {
            for ext in extensions {
                if let Extension::TokenMetadata(_token_metadata) = ext {
                    return SolanaMapper::map_token_data_spl_token_2022(self.get_chain(), token_id, &token_info);
                }
            }
        }

        let metadata = self.get_metaplex_metadata(&token_id).await?;
        SolanaMapper::map_token_data(self.get_chain(), token_id, &token_info, &metadata)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() >= 40 && token_id.len() <= 60 && bs58::decode(token_id).into_vec().is_ok()
    }
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::create_test_client;
    use primitives::{AssetType, Chain};

    #[tokio::test]
    async fn test_get_token_data_usdc_spl_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();

        let asset = client.get_token_data(usdc_mint.clone()).await?;

        assert_eq!(asset.chain, Chain::Solana);
        assert_eq!(asset.token_id, Some(usdc_mint));
        assert_eq!(asset.symbol, "USDC");
        assert_eq!(asset.name, "USD Coin");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.asset_type, AssetType::TOKEN);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_token_data_spl_token_2022() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let spl2022_mint = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo".to_string();

        let asset = client.get_token_data(spl2022_mint.clone()).await?;

        assert_eq!(asset.chain, Chain::Solana);
        assert_eq!(asset.token_id, Some(spl2022_mint));
        assert_eq!(asset.symbol, "PYUSD");
        assert_eq!(asset.name, "PayPal USD");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.asset_type, AssetType::TOKEN);

        Ok(())
    }
}
