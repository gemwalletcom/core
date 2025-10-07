use super::{PancakeSwapAptos, client::PancakeSwapAptosClient};
use crate::alien::{AlienClient, AlienProvider};
use primitives::Chain;
use std::sync::Arc;

impl PancakeSwapAptos<AlienClient> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Aptos).expect("Failed to get Aptos endpoint");
        let client = PancakeSwapAptosClient::new(AlienClient::new(endpoint, rpc_provider));
        Self::with_client(client)
    }
}
