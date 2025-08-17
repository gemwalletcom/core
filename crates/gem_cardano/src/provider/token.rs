use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainToken for CardanoClient<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        Err("Cardano token data not implemented".into())
    }
}
