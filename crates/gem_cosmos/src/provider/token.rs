use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainToken for CosmosClient<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        Err(format!("Token data for {} not implemented", token_id).into())
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("ibc/")
    }
}