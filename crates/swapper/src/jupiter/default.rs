use super::{client::JupiterClient, provider::JUPITER_API_URL, provider::Jupiter};
use crate::alien::{RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::Chain;
use std::sync::Arc;

impl Jupiter<RpcClient, RpcClient> {
    pub fn new(provider: Arc<dyn RpcProvider>) -> Self {
        let http_client = JupiterClient::new(RpcClient::new(JUPITER_API_URL.into(), provider.clone()));
        let solana_endpoint = provider
            .get_endpoint(Chain::Solana)
            .expect("Failed to get Solana endpoint for Jupiter provider");
        let rpc_client = JsonRpcClient::new(RpcClient::new(solana_endpoint, provider));
        Self::with_clients(http_client, rpc_client)
    }
}
