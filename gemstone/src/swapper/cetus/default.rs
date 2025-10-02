use super::{
    api::client::{CETUS_API_URL, CetusClient},
    provider::Cetus,
};
use crate::{
    network::{AlienClient, AlienProvider},
    sui::rpc::SuiClient,
    swapper::Swapper,
};
use std::sync::Arc;

impl Cetus<AlienClient> {
    pub fn new(rpc_provider: Arc<dyn AlienProvider>) -> Self {
        let http_client = CetusClient::new(AlienClient::new(CETUS_API_URL.into(), rpc_provider.clone()));
        let sui_client = Arc::new(SuiClient::new(rpc_provider));
        Self::with_clients(http_client, sui_client)
    }
}

impl Cetus<AlienClient> {
    pub fn boxed(rpc_provider: Arc<dyn AlienProvider>) -> Box<dyn Swapper> {
        Box::new(Self::new(rpc_provider))
    }
}
