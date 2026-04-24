use std::sync::Arc;

use gem_client::ReqwestClient;
use gem_ton::rpc::client::TonClient;

use crate::config::NFTProviderConfig;
use crate::provider::NFTProvider;
use crate::providers::magiceden;
use crate::providers::opensea;
use crate::providers::{MagicEdenEvmClient, MagicEdenSolanaClient, OpenSeaClient};

pub struct NFTProviderFactory;

impl NFTProviderFactory {
    pub fn new_providers(config: NFTProviderConfig) -> Vec<Arc<dyn NFTProvider>> {
        let opensea_client = opensea::create_client(&config.opensea_key);
        let magiceden_client = magiceden::create_client(&config.magiceden_key);
        let ton_client = ReqwestClient::new(config.ton_url, reqwest::Client::new());

        vec![
            Arc::new(OpenSeaClient::new(opensea_client)),
            Arc::new(MagicEdenSolanaClient::new(magiceden_client.clone())),
            Arc::new(MagicEdenEvmClient::new(magiceden_client)),
            Arc::new(TonClient::new(ton_client)),
        ]
    }
}
