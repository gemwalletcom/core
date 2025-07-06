use rand::{rng, Rng};
use settings::Settings;
use settings_chain::{ProviderConfig, ProviderFactory};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use gem_chain_rpc::{ChainAssetsProvider, ChainBlockProvider, ChainProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use primitives::{node::ChainNode, Asset, AssetBalance, Chain, Node, Transaction};

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
    pub fn new(config: ProviderConfig, proxy_config: ParserProxyUrlConfig) -> Self {
        Self {
            chain: config.chain,
            providers: proxy_config
                .urls
                .clone()
                .into_iter()
                .map(|url| ProviderFactory::new_provider(config.with_url(&url)))
                .collect(),
            providers_urls: proxy_config.urls,
            provider_current_index: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new_from_nodes(settings: &Settings, chain: Chain, nodes: Vec<ChainNode>) -> ParserProxy {
        let mut nodes_map: HashMap<String, Vec<Node>> = HashMap::new();
        nodes.into_iter().for_each(|node| {
            nodes_map.entry(node.chain.clone()).or_default().push(node.node);
        });

        let url_type = ProviderFactory::url(chain, settings);
        let node_type = ProviderFactory::get_node_type(url_type.clone());
        let url = url_type.get_url();

        let node_urls = nodes_map
            .clone()
            .get(chain.as_ref())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|x| x.node_type == node_type)
            .map(|x| x.url)
            .collect::<Vec<String>>();

        let node_urls = if node_urls.is_empty() { vec![url.clone()] } else { node_urls };
        let config = ParserProxyUrlConfig { urls: node_urls };
        ParserProxy::new(
            ProviderConfig::new(
                chain,
                &url,
                node_type,
                settings.alchemy.key.secret.as_str(),
                settings.ankr.key.secret.as_str(),
                settings.trongrid.key.secret.as_str(),
            ),
            config,
        )
    }

    fn handle_error(&self, error: Box<dyn Error + Send + Sync>) -> Box<dyn Error + Send + Sync> {
        if self.providers.len() < 2 {
            return error;
        }
        let current_index = *self.provider_current_index.lock().unwrap();
        let new_index = rng().random_range(0..self.providers.len());
        //TODO: Ensure it's not the same as current index

        println!(
            "parser proxy switching for chain: {}, from: {}, to: {}, error: {}",
            self.chain, self.providers_urls[current_index], self.providers_urls[new_index], error
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

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_transactions(block_number).await {
            Ok(txs) => Ok(txs),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainTokenDataProvider for ParserProxy {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_token_data(token_id).await {
            Ok(asset) => Ok(asset),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainAssetsProvider for ParserProxy {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_assets_balances(address).await {
            Ok(balances) => Ok(balances),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainTransactionsProvider for ParserProxy {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_transactions_by_address(address).await {
            Ok(txs) => Ok(txs),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}
