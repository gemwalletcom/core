use rand::{rng, Rng};
use settings::Settings;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use gem_chain_rpc::{ChainBlockProvider, ChainProvider};
use primitives::{node::ChainNode, Chain};

#[derive(Clone, Debug)]
pub struct ParserProxyUrlConfig {
    pub urls: Vec<String>,
}

pub struct ParserProxy {
    pub chain: Chain,
    pub providers: Vec<Box<dyn ChainProvider>>,
    pub providers_urls: Vec<String>,
    provider_current_index: Arc<Mutex<usize>>,
}
impl ParserProxy {
    pub fn new(chain: Chain, config: ParserProxyUrlConfig) -> Self {
        Self {
            chain,
            providers: config.urls.clone().into_iter().map(|x| ParserProxy::new_provider(chain, &x)).collect(),
            providers_urls: config.urls,
            provider_current_index: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new_from_nodes(settings: &Settings, chain: Chain, nodes: Vec<ChainNode>) -> Self {
        let mut nodes_map: HashMap<String, Vec<String>> = HashMap::new();
        nodes.into_iter().for_each(|node| {
            nodes_map.entry(node.chain.clone()).or_default().push(node.node.url);
        });

        let node_urls = nodes_map.clone().get(chain.as_ref()).cloned().unwrap_or_default();

        let url = settings_chain::ProviderFactory::url(chain, settings);
        let node_urls = if node_urls.is_empty() { vec![url.to_string()] } else { node_urls };
        let config = ParserProxyUrlConfig { urls: node_urls };

        Self::new(chain, config)
    }

    // Support ChainBlockProvider once trait_upcasting is enabled
    pub fn new_provider(chain: Chain, url: &str) -> Box<dyn ChainProvider> {
        settings_chain::ProviderFactory::new_provider(chain, url)
    }

    fn handle_error(&self, error: Box<dyn Error + Send + Sync>) -> Box<dyn Error + Send + Sync> {
        println!("parser proxy error: {}", error);

        if self.providers.len() < 2 {
            return error;
        }
        let current_index = *self.provider_current_index.lock().unwrap();
        let new_index = rng().random_range(0..self.providers.len());
        //TODO: Ensure it's not the same as current index

        println!(
            "parser proxy switching for chain: {}, from: {}, to: {}",
            self.chain, self.providers_urls[current_index], self.providers_urls[new_index]
        );

        *self.provider_current_index.lock().unwrap() = new_index;

        error
    }
}

#[async_trait]
impl ChainBlockProvider for ParserProxy {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_latest_block().await {
            Ok(block) => Ok(block),
            Err(err) => Err(self.handle_error(err)),
        }
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_transactions(block_number).await {
            Ok(txs) => Ok(txs),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}
