use std::sync::Arc;

use crate::config::NFTProviderConfig;
use crate::provider::NFTProvider;
use crate::providers::{MagicEdenClient, OpenSeaClient};

pub struct NFTProviderFactory;

impl NFTProviderFactory {
    pub fn new_providers(config: NFTProviderConfig) -> Vec<Arc<dyn NFTProvider>> {
        vec![
            Arc::new(OpenSeaClient::new(&config.opensea_key)),
            Arc::new(MagicEdenClient::new(&config.magiceden_key)),
        ]
    }
}
