use std::error::Error;

use primitives::Chain;
use settings_chain::{ProviderConfig, ProviderFactory};

pub struct ChainClient {
    chain: Chain,
    url: String,
}

impl ChainClient {
    pub fn new(chain: Chain, url: String) -> Self {
        Self { chain, url }
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let config = ProviderConfig::new_url(self.chain, &self.url);
        Ok(ProviderFactory::new_provider(config).get_latest_block().await? as u64)
    }
}
