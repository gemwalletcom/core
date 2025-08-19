#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainToken;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::Asset;
#[cfg(feature = "rpc")]
use std::error::Error;

#[cfg(feature = "rpc")]
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client> ChainToken for SuiClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        self.get_token_data(token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        self.is_token_address(token_id)
    }
}