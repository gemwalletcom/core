use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::Client;
use std::error::Error;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainState for BitcoinClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let block = self.get_block_info(1).await?;
        block.previous_block_hash.ok_or_else(|| "Unable to get block hash".into())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let node_info = self.get_node_info().await?;
        Ok(node_info.blockbook.best_height)
    }
}
