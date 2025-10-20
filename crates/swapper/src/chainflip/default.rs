use super::{
    broker::{BrokerClient, build_broker_path},
    client::ChainflipClient,
    provider::ChainflipProvider,
};
use crate::alien::{RpcClient, RpcProvider};
use gem_jsonrpc::client::JsonRpcClient;
use std::sync::Arc;

const CHAINFLIP_API_URL: &str = "https://api.gemwallet.com/swap/chainflip-swap";
const CHAINFLIP_BROKER_URL: &str = "https://api.gemwallet.com/swap/chainflip-broker";

impl ChainflipProvider<RpcClient, RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let api_client = RpcClient::new(CHAINFLIP_API_URL.into(), rpc_provider.clone());
        let chainflip_client = ChainflipClient::new(api_client.clone());

        let broker_endpoint = build_broker_path(CHAINFLIP_BROKER_URL);
        let broker_client = BrokerClient::new(JsonRpcClient::new(RpcClient::new(broker_endpoint, rpc_provider.clone())));

        Self::with_clients(chainflip_client, broker_client, rpc_provider)
    }
}
