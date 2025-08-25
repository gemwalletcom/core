use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainToken;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::{Asset, AssetId, AssetType};

use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainToken for SuiClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let metadata = self.get_coin_metadata(token_id.clone()).await?;
        let metadata: crate::models::rpc::CoinMetadata = metadata.into();

        let asset = Asset::new(
            AssetId::from_token(self.chain, &token_id),
            metadata.name,
            metadata.symbol,
            metadata.decimals,
            AssetType::TOKEN,
        );
        Ok(asset)
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        self.is_token_address(token_id)
    }
}
