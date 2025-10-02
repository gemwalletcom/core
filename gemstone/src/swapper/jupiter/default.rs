use super::{client::JupiterClient, provider::JUPITER_API_URL, provider::Jupiter};
use crate::network::{AlienClient, AlienProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::Chain;
use std::sync::Arc;

impl Jupiter<AlienClient, AlienClient> {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        let http_client = JupiterClient::new(AlienClient::new(JUPITER_API_URL.into(), provider.clone()));
        let solana_endpoint = provider
            .get_endpoint(Chain::Solana)
            .expect("Failed to get Solana endpoint for Jupiter provider");
        let rpc_client = JsonRpcClient::new(AlienClient::new(solana_endpoint, provider));
        Self::with_clients(http_client, rpc_client)
    }
}
