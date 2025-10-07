use super::{ThorChain, client::ThorChainSwapClient};
use crate::alien::{AlienClient, AlienProvider};
use primitives::Chain;
use std::sync::Arc;

impl ThorChain<AlienClient> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Thorchain).expect("Failed to get Thorchain endpoint");
        let swap_client = ThorChainSwapClient::new(AlienClient::new(endpoint, rpc_provider.clone()));
        Self::with_client(swap_client, rpc_provider)
    }
}
