use rand::{Rng, rng};
use settings::Settings;
use settings_chain::{ProviderConfig, ProviderFactory};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chain_traits::*;
use primitives::{Chain, Node, Transaction, TransactionStateRequest, TransactionUpdate, node::ChainNode};

#[derive(Clone, Debug)]
pub struct ParserProxyUrlConfig {
    pub urls: Vec<String>,
}

pub struct ParserProxy {
    pub chain: Chain,
    pub providers: Vec<Box<dyn ChainTraits>>,
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

        let (url_type, archive_url_type) = ProviderFactory::url(chain, settings);
        let node_type = ProviderFactory::get_node_type(url_type.clone());

        let url = url_type.get_url().unwrap_or_default();
        let archive_url = archive_url_type.unwrap_or(url_type.clone()).get_archive_url().unwrap_or_default();

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
                &archive_url,
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

impl ChainProvider for ParserProxy {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

#[async_trait]
impl ChainState for ParserProxy {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_chain_id().await {
            Ok(chain_id) => Ok(chain_id),
            Err(err) => Err(self.handle_error(err)),
        }
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_block_latest_number().await {
            Ok(block) => Ok(block),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainTransactions for ParserProxy {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_transactions_by_block(block).await {
            Ok(txs) => Ok(txs),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainTransactionState for ParserProxy {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let provider_index = *self.provider_current_index.lock().unwrap();
        match self.providers[provider_index].get_transaction_status(request).await {
            Ok(status) => Ok(status),
            Err(err) => Err(self.handle_error(err)),
        }
    }
}

#[async_trait]
impl ChainBalances for ParserProxy {}

#[async_trait]
impl ChainStaking for ParserProxy {}

#[async_trait]
impl ChainAccount for ParserProxy {}

#[async_trait]
impl ChainPerpetual for ParserProxy {}

#[async_trait]
impl ChainToken for ParserProxy {}

#[async_trait]
impl ChainTransactionLoad for ParserProxy {}

#[async_trait]
impl ChainAddressStatus for ParserProxy {}

impl ChainTraits for ParserProxy {}
