use super::{PancakeSwapAptos, client::PancakeSwapAptosClient};
use crate::alien::{RpcClient, RpcProvider};
use primitives::Chain;
use std::sync::Arc;

impl PancakeSwapAptos<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let endpoint = rpc_provider.get_endpoint(Chain::Aptos).expect("Failed to get Aptos endpoint");
        let client = PancakeSwapAptosClient::new(RpcClient::new(endpoint, rpc_provider));
        Self::with_client(client)
    }
}
