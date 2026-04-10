use super::{broker::BrokerClient, client::ChainflipClient, provider::ChainflipProvider};
use crate::{
    alien::{RpcClient, RpcProvider},
    config::get_swap_api_url,
};
use gem_jsonrpc::client::JsonRpcClient;
use std::sync::Arc;

impl ChainflipProvider<RpcClient, RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let api_url = get_swap_api_url("chainflip-swap");
        let broker_url = get_swap_api_url("chainflip-broker/rpc");

        let api_client = RpcClient::new(api_url, rpc_provider.clone());
        let chainflip_client = ChainflipClient::new(api_client.clone());

        let broker_client = BrokerClient::new(JsonRpcClient::new(RpcClient::new(broker_url, rpc_provider.clone())));

        Self::with_clients(chainflip_client, broker_client, rpc_provider)
    }
}
