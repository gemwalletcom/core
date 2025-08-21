use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainState for StellarClient<C> {
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.ingest_latest_ledger as u64)
    }

    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_status().await?.network_passphrase)
    }
}
