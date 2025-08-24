use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use crate::rpc::client::XRPClient;
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainState for XRPClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok("".to_string())
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_ledger_current().await?.ledger_current_index as u64)
    }
}
