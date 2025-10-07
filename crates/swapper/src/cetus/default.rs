use super::{
    api::client::{CETUS_API_URL, CetusClient},
    provider::Cetus,
};
use crate::{
    Swapper,
    alien::{RpcClient, RpcProvider},
    client_factory::create_client_with_chain,
};
use gem_sui::SuiClient;
use primitives::Chain;
use std::sync::Arc;

impl Cetus<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let http_client = CetusClient::new(RpcClient::new(CETUS_API_URL.into(), rpc_provider.clone()));
        let sui_client = Arc::new(SuiClient::new(create_client_with_chain(rpc_provider.clone(), Chain::Sui)));
        Self::with_clients(http_client, sui_client)
    }
}

impl Cetus<RpcClient> {
    pub fn boxed(rpc_provider: Arc<dyn RpcProvider>) -> Box<dyn Swapper> {
        Box::new(Self::new(rpc_provider))
    }
}
