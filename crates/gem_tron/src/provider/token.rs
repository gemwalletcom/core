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