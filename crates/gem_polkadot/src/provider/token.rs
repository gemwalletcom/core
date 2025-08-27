use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainToken for PolkadotClient<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support tokens".into())
    }

    fn get_is_token_address(&self, _token_id: &str) -> bool {
        false
    }
}
