use super::{
    broker::{BrokerClient, build_broker_path},
    client::ChainflipClient,
    provider::ChainflipProvider,
};
use crate::{
    alien::{RpcClient, RpcProvider},
    config::get_swap_api_url,
};
use gem_jsonrpc::client::JsonRpcClient;
use std::sync::Arc;

impl ChainflipProvider<RpcClient, RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let api_url = get_swap_api_url("chainflip-swap");
        let broker_url = get_swap_api_url("chainflip-broker");

        let api_client = RpcClient::new(api_url, rpc_provider.clone());
        let chainflip_client = ChainflipClient::new(api_client.clone());

        let broker_endpoint = build_broker_path(&broker_url);
        let broker_client = BrokerClient::new(JsonRpcClient::new(RpcClient::new(broker_endpoint, rpc_provider.clone())));

        Self::with_clients(chainflip_client, broker_client, rpc_provider)
    }
}
