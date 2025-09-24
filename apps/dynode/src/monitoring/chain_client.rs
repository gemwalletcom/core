use std::time::Instant;

use primitives::{Chain, NodeStatusState};
use settings_chain::{ProviderConfig, ProviderFactory};

use super::sync::NodeStatusObservation;
use crate::config::Url;

pub struct ChainClient {
    config: ProviderConfig,
    url: Url,
}

impl ChainClient {
    pub fn new(chain: Chain, url: Url) -> Self {
        let config = ProviderConfig::new(chain, &url.url, "", "", "");
        Self { config, url }
    }

    pub async fn fetch_status(&self) -> NodeStatusObservation {
        let started_at = Instant::now();
        let state = match ProviderFactory::new_provider(self.config.clone()).get_node_status().await {
            Ok(status) => NodeStatusState::healthy(status),
            Err(err) => NodeStatusState::error(err.to_string()),
        };

        NodeStatusObservation::new(self.url.clone(), state, started_at.elapsed())
    }
}
