use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::TonClient;

#[async_trait]
impl<C: Client> ChainToken for TonClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        self.get_token_data(token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("EQ") && token_id.len() >= 40 && token_id.len() <= 60
    }
}
