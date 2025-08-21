use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainState for SolanaClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_genesis_hash().await
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_slot().await
    }
}
