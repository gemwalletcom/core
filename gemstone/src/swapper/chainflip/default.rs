use super::{
    broker::{BrokerClient, build_broker_path},
    client::ChainflipClient,
    provider::ChainflipProvider,
};
use crate::network::{AlienClient, AlienProvider};
use gem_jsonrpc::client::JsonRpcClient;
use std::sync::Arc;

const CHAINFLIP_API_URL: &str = "https://chainflip-swap.chainflip.io";
const CHAINFLIP_BROKER_URL: &str = "https://chainflip-broker.io";

impl ChainflipProvider<AlienClient, AlienClient> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let api_client = AlienClient::new(CHAINFLIP_API_URL.into(), rpc_provider.clone());
        let chainflip_client = ChainflipClient::new(api_client.clone());

        let broker_endpoint = build_broker_path(CHAINFLIP_BROKER_URL);
        let broker_client = BrokerClient::new(JsonRpcClient::new(AlienClient::new(broker_endpoint, rpc_provider.clone())));

        Self::with_clients(chainflip_client, broker_client, rpc_provider)
    }
}
