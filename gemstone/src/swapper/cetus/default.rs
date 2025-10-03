use super::{
    api::client::{CETUS_API_URL, CetusClient},
    provider::Cetus,
};
use crate::{
    network::{AlienClient, AlienProvider, jsonrpc_client_with_chain},
    swapper::Swapper,
};
use gem_sui::rpc::SuiClient;
use primitives::Chain;
use std::sync::Arc;

impl Cetus<AlienClient> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let http_client = CetusClient::new(AlienClient::new(CETUS_API_URL.into(), rpc_provider.clone()));
        let sui_rpc_client = jsonrpc_client_with_chain(rpc_provider.clone(), Chain::Sui);
        let sui_client = Arc::new(SuiClient::new(sui_rpc_client));
        Self::with_clients(http_client, sui_client)
    }
}

impl Cetus<AlienClient> {
    pub fn boxed(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
        Box::new(Self::new(rpc_provider))
    }
}

pub fn new_cetus(rpc_provider: Arc<dyn AlienProvider>) -> Cetus<AlienClient> {
    Cetus::<AlienClient>::new(rpc_provider)
}

pub fn boxed_cetus(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
    Cetus::<AlienClient>::boxed(rpc_provider)
}
